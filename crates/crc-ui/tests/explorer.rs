use std::path::PathBuf;

use crc_ui::view::explorer::tree;

fn paths(list: &[&str]) -> Vec<PathBuf> {
    list.iter().map(PathBuf::from).collect()
}

fn shape(entries: &[crc_ui::view::FileEntry]) -> Vec<String> {
    entries
        .iter()
        .map(|entry| {
            format!(
                "{}{}{}",
                "  ".repeat(entry.depth),
                entry.name,
                if entry.is_dir { "/" } else { "" }
            )
        })
        .collect()
}

#[test]
fn a_flat_list_of_paths_becomes_a_tree() {
    let entries = tree(&paths(&["src/main.rs", "src/lib.rs", "Cargo.toml"]), 50);

    assert_eq!(
        shape(&entries),
        vec!["src/", "  lib.rs", "  main.rs", "Cargo.toml"]
    );
}

#[test]
fn a_folder_is_announced_once_however_many_files_it_holds() {
    let entries = tree(&paths(&["src/a.rs", "src/b.rs", "src/c.rs"]), 50);

    assert_eq!(
        entries.iter().filter(|entry| entry.is_dir).count(),
        1,
        "src is listed once, not once per file"
    );
}

#[test]
fn nesting_carries_the_right_depth() {
    let entries = tree(&paths(&["crates/core/src/fs/ops.rs"]), 50);

    assert_eq!(
        shape(&entries),
        vec![
            "crates/",
            "  core/",
            "    src/",
            "      fs/",
            "        ops.rs"
        ]
    );
}

#[test]
fn folders_come_before_the_files_beside_them() {
    let entries = tree(&paths(&["README.md", "src/main.rs"]), 50);

    assert_eq!(
        shape(&entries),
        vec!["src/", "  main.rs", "README.md"],
        "a folder sorts above a loose file at the same level"
    );
}

#[test]
fn siblings_are_alphabetical_and_case_does_not_split_them() {
    let entries = tree(
        &paths(&["src/Zebra.rs", "src/apple.rs", "src/Mango.rs"]),
        50,
    );

    assert_eq!(
        shape(&entries),
        vec!["src/", "  apple.rs", "  Mango.rs", "  Zebra.rs"]
    );
}

#[test]
fn leaving_one_branch_for_another_closes_the_first() {
    let entries = tree(&paths(&["a/deep/one.rs", "b/two.rs"]), 50);

    assert_eq!(
        shape(&entries),
        vec!["a/", "  deep/", "    one.rs", "b/", "  two.rs"]
    );
}

#[test]
fn every_file_carries_the_path_that_opens_it() {
    let entries = tree(&paths(&["src/main.rs", "Cargo.toml"]), 50);

    for entry in &entries {
        assert!(
            entry.path.is_some(),
            "{} has no path, so nothing can act on it",
            entry.name
        );
    }

    let main = entries.iter().find(|e| e.name == "main.rs").unwrap();
    assert_eq!(
        main.path.as_deref(),
        Some(PathBuf::from("src/main.rs").as_path())
    );
}

#[test]
fn a_folder_carries_its_own_path_so_it_can_be_acted_on() {
    let entries = tree(&paths(&["src/view/shell.rs", "src/main.rs"]), 50);

    let src = entries.iter().find(|e| e.name == "src").unwrap();
    let view = entries.iter().find(|e| e.name == "view").unwrap();

    assert!(src.is_dir && view.is_dir);
    assert_eq!(src.path.as_deref(), Some(PathBuf::from("src").as_path()));
    assert_eq!(
        view.path.as_deref(),
        Some(PathBuf::from("src/view").as_path()),
        "a nested folder knows the whole way down to it"
    );
}

#[test]
fn the_limit_stops_the_listing_rather_than_truncating_a_row() {
    let entries = tree(&paths(&["src/a.rs", "src/b.rs", "src/c.rs", "src/d.rs"]), 3);

    assert_eq!(entries.len(), 3);
    assert_eq!(shape(&entries), vec!["src/", "  a.rs", "  b.rs"]);
}

#[test]
fn nothing_in_nothing_out() {
    assert!(tree(&[], 50).is_empty());
}

#[test]
fn a_bare_file_name_needs_no_folder() {
    let entries = tree(&paths(&["LICENSE"]), 50);

    assert_eq!(shape(&entries), vec!["LICENSE"]);
    assert_eq!(entries[0].depth, 0);
}
