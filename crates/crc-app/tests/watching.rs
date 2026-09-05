use std::path::Path;
use std::time::{Duration, Instant};

use crc_app::Session;

fn scratch(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("crc-watch-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("a folder to work in");
    root
}

fn settle(session: &mut Session, patience: Duration) -> Vec<String> {
    let started = Instant::now();
    while started.elapsed() < patience {
        let touched = session.reload_from_disk();
        if !touched.is_empty() {
            return touched;
        }
        std::thread::sleep(Duration::from_millis(60));
    }
    Vec::new()
}

#[test]
fn a_file_changed_underneath_is_picked_up() {
    let root = scratch("picked-up");
    std::fs::write(root.join("a.rs"), "fn one() {}\n").unwrap();

    let mut session = Session::open(&root).expect("a workspace");
    session.open_file(Path::new("a.rs")).expect("the file opens");
    assert_eq!(session.document().unwrap().text(), "fn one() {}\n");

    std::fs::write(root.join("a.rs"), "fn two() {}\n").unwrap();
    let touched = settle(&mut session, Duration::from_secs(15));

    assert!(!touched.is_empty(), "the change was never noticed");
    assert_eq!(session.document().unwrap().text(), "fn two() {}\n");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn unsaved_work_is_never_clobbered_by_the_file_on_disk() {
    let root = scratch("unsaved");
    std::fs::write(root.join("b.rs"), "fn one() {}\n").unwrap();

    let mut session = Session::open(&root).expect("a workspace");
    session.open_file(Path::new("b.rs")).expect("the file opens");

    session.document().unwrap().insert("// mine\n");
    let mine = session.document().unwrap().text().to_string();
    assert!(session.document().unwrap().is_dirty());

    std::fs::write(root.join("b.rs"), "fn theirs() {}\n").unwrap();
    let _ = settle(&mut session, Duration::from_secs(3));

    assert_eq!(
        session.document().unwrap().text(),
        mine,
        "an unsaved buffer must win over the file on disk"
    );
    assert!(session.document().unwrap().is_dirty());

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_file_written_with_the_same_content_changes_nothing() {
    let root = scratch("same");
    std::fs::write(root.join("c.rs"), "fn one() {}\n").unwrap();

    let mut session = Session::open(&root).expect("a workspace");
    session.open_file(Path::new("c.rs")).expect("the file opens");

    std::fs::write(root.join("c.rs"), "fn one() {}\n").unwrap();

    let started = Instant::now();
    let mut touched = Vec::new();
    while started.elapsed() < Duration::from_secs(3) {
        touched.extend(session.reload_from_disk());
        std::thread::sleep(Duration::from_millis(60));
    }

    assert!(
        touched.is_empty(),
        "rewriting the same bytes is not a change worth reporting"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn nothing_is_reported_when_nothing_happened() {
    let root = scratch("quiet");
    std::fs::write(root.join("d.rs"), "fn one() {}\n").unwrap();

    let mut session = Session::open(&root).expect("a workspace");
    session.open_file(Path::new("d.rs")).expect("the file opens");

    std::thread::sleep(Duration::from_millis(300));
    assert!(session.reload_from_disk().is_empty());

    let _ = std::fs::remove_dir_all(&root);
}
