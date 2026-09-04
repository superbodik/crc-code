use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use portable_pty::{CommandBuilder, MasterPty, PtySize, native_pty_system};

use crate::screen::{Cell, Ink, Row, Screen};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shell {
    pub program: String,
    pub arguments: Vec<String>,
}

impl Shell {
    pub fn preferred() -> Self {
        if cfg!(windows) {
            if let Some(pwsh) = std::env::var_os("ProgramFiles").and_then(|dir| {
                let candidate = Path::new(&dir).join("PowerShell/7/pwsh.exe");
                candidate.exists().then_some(candidate)
            }) {
                return Self {
                    program: pwsh.to_string_lossy().into_owned(),
                    arguments: vec!["-NoLogo".to_string()],
                };
            }
            return Self {
                program: "powershell.exe".to_string(),
                arguments: vec!["-NoLogo".to_string()],
            };
        }

        let program = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        Self {
            program,
            arguments: Vec::new(),
        }
    }
}

type Ink0 = Box<dyn Write + Send>;

pub struct Terminal {
    master: Box<dyn MasterPty + Send>,
    writer: Arc<Mutex<Ink0>>,
    parser: Arc<Mutex<vt100::Parser>>,
    alive: Arc<AtomicBool>,
    revision: Arc<AtomicU64>,
    rows: u16,
    columns: u16,
}

impl Terminal {
    pub fn spawn(shell: &Shell, cwd: &Path, rows: u16, columns: u16) -> Result<Self> {
        let rows = rows.max(1);
        let columns = columns.max(1);

        let system = native_pty_system();
        let pair = system
            .openpty(PtySize {
                rows,
                cols: columns,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("the system would not open a pseudo terminal")?;

        let mut command = CommandBuilder::new(&shell.program);
        for argument in &shell.arguments {
            command.arg(argument);
        }
        command.cwd(cwd);
        command.env("TERM", "xterm-256color");

        let mut child = pair
            .slave
            .spawn_command(command)
            .with_context(|| format!("could not start {}", shell.program))?;
        drop(pair.slave);

        let writer: Arc<Mutex<Ink0>> = Arc::new(Mutex::new(
            pair.master
                .take_writer()
                .context("the terminal has no way in")?,
        ));
        let mut reader = pair
            .master
            .try_clone_reader()
            .context("the terminal has no way out")?;

        let parser = Arc::new(Mutex::new(vt100::Parser::new(rows, columns, 4000)));
        let alive = Arc::new(AtomicBool::new(true));
        let revision = Arc::new(AtomicU64::new(0));

        let feed = Arc::clone(&parser);
        let running = Arc::clone(&alive);
        let answer = Arc::clone(&writer);
        let counter = Arc::clone(&revision);

        std::thread::Builder::new()
            .name("crc-term-reader".to_string())
            .spawn(move || {
                let mut buffer = [0u8; 8192];
                let mut tail = Vec::new();

                loop {
                    match reader.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(read) => {
                            let chunk = &buffer[..read];
                            if let Ok(mut parser) = feed.lock() {
                                parser.process(chunk);
                            }

                            counter.fetch_add(1, Ordering::Relaxed);

                            tail.extend_from_slice(chunk);
                            let (asked, consumed) = answers(&tail);
                            tail.drain(0..consumed);
                            if tail.len() > 32 {
                                let cut = tail.len() - 32;
                                tail.drain(0..cut);
                            }

                            let cursor = feed
                                .lock()
                                .map(|parser| parser.screen().cursor_position())
                                .unwrap_or((0, 0));

                            for reply in asked {
                                let bytes = match reply {
                                    Query::CursorPosition => {
                                        format!("\x1b[{};{}R", cursor.0 + 1, cursor.1 + 1)
                                            .into_bytes()
                                    }
                                    Query::DeviceAttributes => b"\x1b[?1;0c".to_vec(),
                                };
                                if let Ok(mut writer) = answer.lock() {
                                    let _ = writer.write_all(&bytes);
                                    let _ = writer.flush();
                                }
                            }
                        }
                        Err(error) => {
                            tracing::debug!("terminal reader stopped: {error}");
                            break;
                        }
                    }
                }
                running.store(false, Ordering::Relaxed);
            })
            .context("could not start the terminal reader")?;

        let watching = Arc::clone(&alive);
        std::thread::Builder::new()
            .name("crc-term-wait".to_string())
            .spawn(move || {
                let _ = child.wait();
                watching.store(false, Ordering::Relaxed);
            })
            .context("could not watch the shell")?;

        Ok(Self {
            master: pair.master,
            writer,
            parser,
            alive,
            revision,
            rows,
            columns,
        })
    }

    pub fn is_alive(&self) -> bool {
        self.alive.load(Ordering::Relaxed)
    }

    pub fn revision(&self) -> u64 {
        self.revision.load(Ordering::Relaxed)
    }

    pub fn size(&self) -> (u16, u16) {
        (self.rows, self.columns)
    }

    pub fn send(&mut self, bytes: &[u8]) {
        let Ok(mut writer) = self.writer.lock() else {
            return;
        };
        if let Err(error) = writer.write_all(bytes) {
            tracing::debug!("terminal would not take input: {error}");
            self.alive.store(false, Ordering::Relaxed);
            return;
        }
        let _ = writer.flush();
    }

    pub fn resize(&mut self, rows: u16, columns: u16) {
        let rows = rows.max(1);
        let columns = columns.max(1);
        if rows == self.rows && columns == self.columns {
            return;
        }

        if let Err(error) = self.master.resize(PtySize {
            rows,
            cols: columns,
            pixel_width: 0,
            pixel_height: 0,
        }) {
            tracing::debug!("terminal would not resize: {error}");
            return;
        }

        if let Ok(mut parser) = self.parser.lock() {
            parser.screen_mut().set_size(rows, columns);
        }
        self.rows = rows;
        self.columns = columns;
    }

    pub fn screen(&self) -> Screen {
        let Ok(parser) = self.parser.lock() else {
            return Screen::default();
        };
        let screen = parser.screen();
        let (rows, columns) = screen.size();

        let mut out = Vec::with_capacity(rows as usize);
        for row in 0..rows {
            let mut cells = Vec::with_capacity(columns as usize);
            for column in 0..columns {
                cells.push(match screen.cell(row, column) {
                    Some(cell) => Cell {
                        text: {
                            let text = cell.contents();
                            if text.trim().is_empty() {
                                " ".to_string()
                            } else {
                                text.to_string()
                            }
                        },
                        foreground: ink(cell.fgcolor()),
                        background: ink(cell.bgcolor()),
                        bold: cell.bold(),
                        inverse: cell.inverse(),
                    },
                    None => Cell::default(),
                });
            }
            out.push(Row { cells });
        }

        Screen {
            rows: out,
            cursor: screen.cursor_position(),
            cursor_visible: !screen.hide_cursor(),
            alive: self.alive.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Query {
    CursorPosition,
    DeviceAttributes,
}

pub fn answers(bytes: &[u8]) -> (Vec<Query>, usize) {
    let mut asked = Vec::new();
    let mut at = 0;
    let mut consumed = 0;

    while at < bytes.len() {
        if bytes[at] != 0x1b {
            at += 1;
            consumed = at;
            continue;
        }

        if at + 1 >= bytes.len() {
            break;
        }
        if bytes[at + 1] != b'[' {
            at += 2;
            consumed = at;
            continue;
        }

        let mut end = at + 2;
        while end < bytes.len() && (bytes[end].is_ascii_digit() || bytes[end] == b';') {
            end += 1;
        }
        if end >= bytes.len() {
            break;
        }

        match bytes[end] {
            b'n' if &bytes[at + 2..end] == b"6" => asked.push(Query::CursorPosition),
            b'c' => asked.push(Query::DeviceAttributes),
            _ => {}
        }
        at = end + 1;
        consumed = at;
    }

    (asked, consumed)
}

fn ink(colour: vt100::Color) -> Ink {
    match colour {
        vt100::Color::Default => Ink::Default,
        vt100::Color::Idx(index) => Ink::Indexed(index),
        vt100::Color::Rgb(r, g, b) => Ink::Rgb(r, g, b),
    }
}
