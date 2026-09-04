use std::time::{Duration, Instant};

use crc_term::{Shell, Terminal};

fn wait_for(terminal: &Terminal, needle: &str, patience: Duration) -> bool {
    let started = Instant::now();
    while started.elapsed() < patience {
        let screen = terminal.screen();
        let text: String = screen
            .rows
            .iter()
            .map(|row| row.text())
            .collect::<Vec<_>>()
            .join("\n");
        if text.contains(needle) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(80));
    }
    false
}

fn open(rows: u16, columns: u16) -> Terminal {
    let cwd = std::env::temp_dir();
    Terminal::spawn(&Shell::preferred(), &cwd, rows, columns).expect("a shell")
}

#[test]
fn a_shell_starts_and_stays_alive() {
    let terminal = open(24, 80);

    assert!(terminal.is_alive());
    assert_eq!(terminal.size(), (24, 80));
}

#[test]
fn the_screen_has_the_shape_it_was_given() {
    let terminal = open(12, 40);
    std::thread::sleep(Duration::from_millis(400));

    let (rows, columns) = terminal.screen().size();
    assert_eq!(rows, 12);
    assert_eq!(columns, 40);
}

#[test]
fn what_is_typed_comes_back_out() {
    let mut terminal = open(24, 80);
    std::thread::sleep(Duration::from_millis(700));

    terminal.send(b"echo crc-lives\r");

    assert!(
        wait_for(&terminal, "crc-lives", Duration::from_secs(20)),
        "the shell never answered"
    );
}

#[test]
fn a_resize_reshapes_the_screen() {
    let mut terminal = open(24, 80);
    std::thread::sleep(Duration::from_millis(500));

    terminal.resize(30, 100);

    assert_eq!(terminal.size(), (30, 100));
    let (rows, columns) = terminal.screen().size();
    assert_eq!(rows, 30);
    assert_eq!(columns, 100);
}

#[test]
fn a_zero_size_is_refused_rather_than_crashing() {
    let mut terminal = open(24, 80);
    terminal.resize(0, 0);

    let (rows, columns) = terminal.size();
    assert!(rows >= 1 && columns >= 1, "a terminal cannot be nothing");
}

#[test]
fn blank_rows_at_the_bottom_are_trimmed_away() {
    let terminal = open(24, 80);
    std::thread::sleep(Duration::from_millis(700));

    let screen = terminal.screen();
    assert!(
        screen.trimmed().len() <= screen.rows.len(),
        "trimming cannot add rows"
    );
}

#[test]
fn a_shell_that_finishes_is_reported_as_gone() {
    let brief = if cfg!(windows) {
        Shell {
            program: "cmd.exe".to_string(),
            arguments: vec!["/c".to_string(), "echo done".to_string()],
        }
    } else {
        Shell {
            program: "/bin/sh".to_string(),
            arguments: vec!["-c".to_string(), "echo done".to_string()],
        }
    };

    let terminal =
        Terminal::spawn(&brief, &std::env::temp_dir(), 24, 80).expect("a short-lived shell");

    let started = Instant::now();
    while terminal.is_alive() && started.elapsed() < Duration::from_secs(20) {
        std::thread::sleep(Duration::from_millis(80));
    }

    assert!(
        !terminal.is_alive(),
        "the editor never noticed the shell finish"
    );
}

#[test]
fn the_editor_answers_the_questions_a_console_asks() {
    use crc_term::session::{Query, answers};

    assert_eq!(answers(b"\x1b[6n"), vec![Query::CursorPosition]);
    assert_eq!(answers(b"\x1b[c"), vec![Query::DeviceAttributes]);
    assert_eq!(answers(b"\x1b[0c"), vec![Query::DeviceAttributes]);
    assert!(answers(b"just some output").is_empty());
    assert!(
        answers(b"\x1b[2J\x1b[H").is_empty(),
        "clearing the screen is not a question"
    );
    assert_eq!(
        answers(b"hello\x1b[6nworld\x1b[c"),
        vec![Query::CursorPosition, Query::DeviceAttributes],
        "every question in a chunk gets an answer"
    );
}
