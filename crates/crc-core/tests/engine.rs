use std::time::Duration;

use crc_core::fs::TextQuery;
use crc_core::{CoreError, Engine, Event, Limits};

fn workspace() -> (tempfile::TempDir, std::sync::Arc<Engine>) {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/main.rs"), "fn main() {\n    greet();\n}\n").unwrap();
    std::fs::write(root.join("src/util.rs"), "pub fn greet() {}\n").unwrap();
    std::fs::write(root.join("README.md"), "# demo\n").unwrap();

    let engine = Engine::open(root, Limits::default()).expect("open workspace");
    (dir, engine)
}

#[tokio::test]
async fn reads_and_writes_a_file() {
    let (_dir, engine) = workspace();

    let doc = engine.read_file("src/main.rs").await.unwrap();
    assert!(doc.text.contains("greet()"));
    assert_eq!(doc.version, 1);
    assert!(!doc.dirty);

    let version = engine
        .write_file("src/main.rs", "fn main() {}\n".into(), Some(doc.version))
        .await
        .unwrap();
    assert_eq!(version, 2);

    let on_disk = std::fs::read_to_string(engine.root().join("src/main.rs")).unwrap();
    assert_eq!(on_disk, "fn main() {}\n");
}

#[tokio::test]
async fn creates_files_and_parent_directories() {
    let (_dir, engine) = workspace();

    engine
        .write_file("a/b/c/new.txt", "hello".into(), None)
        .await
        .unwrap();

    let doc = engine.read_file("a/b/c/new.txt").await.unwrap();
    assert_eq!(doc.text, "hello");
}

#[tokio::test]
async fn rejects_paths_outside_the_workspace() {
    let (_dir, engine) = workspace();

    for escape in [
        "../outside.txt",
        "src/../../outside.txt",
        "src/../../../etc/passwd",
    ] {
        let err = engine.read_file(escape).await.unwrap_err();
        assert!(
            matches!(err, CoreError::EscapesWorkspace(_)),
            "reading {escape} should be refused, got {err:?}"
        );

        let err = engine
            .write_file(escape, "pwned".into(), None)
            .await
            .unwrap_err();
        assert!(
            matches!(err, CoreError::EscapesWorkspace(_)),
            "writing {escape} should be refused, got {err:?}"
        );
    }
}

#[tokio::test]
async fn rejects_absolute_paths_outside_the_workspace() {
    let (_dir, engine) = workspace();
    let outside = tempfile::tempdir().unwrap();
    let target = outside.path().join("secret.txt");
    std::fs::write(&target, "secret").unwrap();

    let err = engine.read_file(&target).await.unwrap_err();
    assert!(matches!(err, CoreError::EscapesWorkspace(_)), "{err:?}");
}

#[tokio::test]
async fn refuses_a_write_against_a_stale_version() {
    let (_dir, engine) = workspace();

    let doc = engine.read_file("README.md").await.unwrap();
    engine
        .write_file(
            "README.md",
            "# edited by the user\n".into(),
            Some(doc.version),
        )
        .await
        .unwrap();

    let err = engine
        .write_file(
            "README.md",
            "# edited by the agent\n".into(),
            Some(doc.version),
        )
        .await
        .unwrap_err();

    assert!(
        matches!(
            err,
            CoreError::VersionConflict {
                expected: 1,
                actual: 2,
                ..
            }
        ),
        "{err:?}"
    );
    let on_disk = std::fs::read_to_string(engine.root().join("README.md")).unwrap();
    assert_eq!(on_disk, "# edited by the user\n");
}

#[tokio::test]
async fn refuses_a_file_over_the_size_limit() {
    let (_dir, engine) = workspace();
    let limits = Limits {
        max_text_bytes: 8,
        ..Limits::default()
    };
    let small = Engine::open(engine.root(), limits).unwrap();

    let err = small.read_file("src/main.rs").await.unwrap_err();
    assert!(matches!(err, CoreError::TooLarge { .. }), "{err:?}");
}

#[tokio::test]
async fn refuses_a_file_that_is_not_utf8() {
    let (_dir, engine) = workspace();
    std::fs::write(engine.root().join("blob.bin"), [0xff, 0xfe, 0x00]).unwrap();

    let err = engine.read_file("blob.bin").await.unwrap_err();
    assert!(matches!(err, CoreError::NotUtf8(_)), "{err:?}");
}

#[tokio::test]
async fn lists_a_directory_with_directories_first() {
    let (_dir, engine) = workspace();

    let entries = engine.list_dir(".").await.unwrap();
    let names: Vec<_> = entries.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, ["src", "README.md"]);
    assert!(entries[0].is_dir);
}

#[tokio::test]
async fn searches_file_contents() {
    let (_dir, engine) = workspace();

    let hits = engine
        .search_text(TextQuery::literal("greet"))
        .await
        .unwrap();
    let paths: Vec<_> = hits
        .iter()
        .map(|h| h.path.to_string_lossy().replace('\\', "/"))
        .collect();
    assert_eq!(paths, ["src/main.rs", "src/util.rs"]);
    assert_eq!(hits[0].matches[0].line, 2);
}

#[tokio::test]
async fn treats_a_literal_query_as_literal() {
    let (_dir, engine) = workspace();
    std::fs::write(engine.root().join("re.txt"), "a.c\nabc\n").unwrap();

    let hits = engine.search_text(TextQuery::literal("a.c")).await.unwrap();
    let lines: Vec<_> = hits
        .iter()
        .flat_map(|h| h.matches.iter().map(|m| m.text.clone()))
        .collect();
    assert_eq!(lines, ["a.c"], "the dot must not match any character");
}

#[tokio::test]
async fn finds_files_by_fuzzy_name() {
    let (_dir, engine) = workspace();

    let found = engine.find_files("mainrs", 10).await.unwrap();
    let first = found
        .first()
        .expect("a match")
        .to_string_lossy()
        .replace('\\', "/");
    assert_eq!(first, "src/main.rs");

    assert!(engine.find_files("zzzz", 10).await.unwrap().is_empty());
}

#[tokio::test]
async fn announces_a_change_made_outside_the_editor() {
    let (_dir, engine) = workspace();
    let mut events = engine.subscribe();

    std::fs::write(engine.root().join("src/util.rs"), "pub fn greet() { }\n").unwrap();

    let seen = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            match events.recv().await.unwrap() {
                Event::FileChanged { path, .. } => {
                    if path.to_string_lossy().contains("util.rs") {
                        return true;
                    }
                }
                _ => continue,
            }
        }
    })
    .await;

    assert!(seen.is_ok(), "the watcher never reported the change");
}
