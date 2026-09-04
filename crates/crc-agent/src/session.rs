use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, TryRecvError, channel};
use std::sync::Arc;

use anyhow::{Context, Result};

use crate::event::{Event, read};

pub const PROGRAM: &str = "claude";

pub fn candidates() -> Vec<String> {
    if cfg!(windows) {
        vec![
            "claude.cmd".to_string(),
            "claude.exe".to_string(),
            "claude.bat".to_string(),
            "claude".to_string(),
        ]
    } else {
        vec!["claude".to_string()]
    }
}

pub fn needs_a_shell(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.eq_ignore_ascii_case("cmd") || extension.eq_ignore_ascii_case("bat"))
        .unwrap_or(false)
}

pub fn locate() -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;

    for folder in std::env::split_paths(&path) {
        for name in candidates() {
            let candidate = folder.join(&name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn launcher(model: Option<&str>) -> Option<Command> {
    let found = locate()?;

    let mut command = if needs_a_shell(&found) {
        let mut shell = Command::new("cmd");
        shell.arg("/c").arg(&found);
        shell
    } else {
        Command::new(&found)
    };

    command.args(arguments(model));
    Some(command)
}

pub fn arguments(model: Option<&str>) -> Vec<String> {
    let mut flags = vec![
        "-p".to_string(),
        "--input-format".to_string(),
        "stream-json".to_string(),
        "--output-format".to_string(),
        "stream-json".to_string(),
        "--verbose".to_string(),
    ];

    if let Some(model) = model {
        flags.push("--model".to_string());
        flags.push(model.to_string());
    }

    flags
}

pub fn ask(text: &str) -> String {
    let message = serde_json::json!({
        "type": "user",
        "message": {
            "role": "user",
            "content": [{ "type": "text", "text": text }]
        }
    });
    format!("{message}\n")
}

pub struct Agent {
    child: Child,
    stdin: Option<ChildStdin>,
    events: Receiver<Event>,
    alive: Arc<AtomicBool>,
}

impl Agent {
    pub fn start(root: &Path, model: Option<&str>) -> Result<Self> {
        let mut command =
            launcher(model).with_context(|| format!("{PROGRAM} is not on the PATH"))?;

        let mut child = command
            .current_dir(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("could not start {PROGRAM}"))?;

        let stdin = child.stdin.take().context("the agent has no way in")?;
        let stdout = child.stdout.take().context("the agent has no way out")?;
        let stderr = child.stderr.take();

        let (sender, events) = channel();
        let alive = Arc::new(AtomicBool::new(true));

        let running = Arc::clone(&alive);
        let post = sender.clone();
        std::thread::Builder::new()
            .name("crc-agent-reader".to_string())
            .spawn(move || {
                for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                    for event in read(&line) {
                        if post.send(event).is_err() {
                            break;
                        }
                    }
                }
                let _ = post.send(Event::Gone);
                running.store(false, Ordering::Relaxed);
            })
            .context("could not start the agent reader")?;

        if let Some(stderr) = stderr {
            let complain = sender;
            std::thread::Builder::new()
                .name("crc-agent-stderr".to_string())
                .spawn(move || {
                    for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                        if line.trim().is_empty() {
                            continue;
                        }
                        if complain.send(Event::Trouble(line)).is_err() {
                            break;
                        }
                    }
                })
                .ok();
        }

        Ok(Self {
            child,
            stdin: Some(stdin),
            events,
            alive,
        })
    }

    pub fn is_alive(&self) -> bool {
        self.alive.load(Ordering::Relaxed)
    }

    pub fn say(&mut self, text: &str) -> Result<()> {
        let Some(stdin) = self.stdin.as_mut() else {
            anyhow::bail!("the agent is no longer listening");
        };
        stdin.write_all(ask(text).as_bytes())?;
        stdin.flush()?;
        Ok(())
    }

    pub fn drain(&mut self) -> Vec<Event> {
        let mut events = Vec::new();
        loop {
            match self.events.try_recv() {
                Ok(event) => events.push(event),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.alive.store(false, Ordering::Relaxed);
                    break;
                }
            }
        }
        events
    }

    pub fn stop(&mut self) {
        self.stdin = None;
        let _ = self.child.kill();
        let _ = self.child.wait();
        self.alive.store(false, Ordering::Relaxed);
    }
}

impl Drop for Agent {
    fn drop(&mut self) {
        self.stop();
    }
}

pub fn installed() -> bool {
    locate().is_some()
}
