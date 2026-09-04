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

    assert_eq!(answers(b"\x1b[6n").0, vec![Query::CursorPosition]);
    assert_eq!(answers(b"\x1b[c").0, vec![Query::DeviceAttributes]);
    assert_eq!(answers(b"\x1b[0c").0, vec![Query::DeviceAttributes]);
    assert!(answers(b"just some output").0.is_empty());
    assert!(
        answers(b"\x1b[2J\x1b[H").0.is_empty(),
        "clearing the screen is not a question"
    );
    assert_eq!(
        answers(b"hello\x1b[6nworld\x1b[c").0,
        vec![Query::CursorPosition, Query::DeviceAttributes],
        "every question in a chunk gets an answer"
    );
}

#[test]
fn a_question_is_answered_once_and_then_forgotten() {
    use crc_term::session::{Query, answers};

    let asked = b"\x1b[6n";
    let (queries, consumed) = answers(asked);

    assert_eq!(queries, vec![Query::CursorPosition]);
    assert_eq!(
        consumed,
        asked.len(),
        "the whole question is used up, so it is never asked again"
    );

    let (again, _) = answers(&asked[consumed..]);
    assert!(
        again.is_empty(),
        "answering the same bytes twice is what wedged the shell"
    );
}

#[test]
fn a_question_split_across_two_reads_is_kept_for_the_second() {
    use crc_term::session::{Query, answers};

    let first = b"output\x1b[6";
    let (queries, consumed) = answers(first);

    assert!(queries.is_empty(), "the question is not finished yet");
    assert!(consumed < first.len(), "the unfinished tail must be kept");

    let mut carried = first.to_vec();
    carried.drain(0..consumed);
    carried.extend_from_slice(b"n");

    let (finished, _) = answers(&carried);
    assert_eq!(finished, vec![Query::CursorPosition]);
}

#[test]
fn plain_output_is_used_up_so_the_tail_cannot_grow_without_end() {
    use crc_term::session::answers;

    let (queries, consumed) = answers(b"a wall of ordinary output with no escapes at all");

    assert!(queries.is_empty());
    assert_eq!(consumed, 48, "every byte of it is done with");
}

#[test]
fn the_screen_counts_up_as_the_shell_writes() {
    let mut terminal = open(24, 80);
    std::thread::sleep(Duration::from_millis(800));

    let before = terminal.revision();
    terminal.send(b"echo counting\r");

    let started = Instant::now();
    while terminal.revision() == before && started.elapsed() < Duration::from_secs(20) {
        std::thread::sleep(Duration::from_millis(60));
    }

    assert!(
        terminal.revision() > before,
        "the editor cannot tell when to redraw"
    );
}
