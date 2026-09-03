use crc_editor::{Document, Documents, Motion};

fn doc(name: &str) -> Document {
    Document::open(name, format!("contents of {name}\n"))
}

fn open(names: &[&str]) -> Documents {
    let mut documents = Documents::new();
    for name in names {
        documents.open(doc(name));
    }
    documents
}

#[test]
fn opening_a_file_makes_it_the_active_one() {
    let documents = open(&["a.rs", "b.rs"]);

    assert_eq!(documents.len(), 2);
    assert_eq!(documents.active_index(), Some(1));
    assert_eq!(
        documents.active().unwrap().path(),
        std::path::Path::new("b.rs")
    );
}

#[test]
fn opening_a_file_that_is_already_open_just_focuses_it() {
    let mut documents = open(&["a.rs", "b.rs", "c.rs"]);

    let index = documents.open(doc("a.rs"));

    assert_eq!(index, 0);
    assert_eq!(documents.len(), 3, "no duplicate tab");
    assert_eq!(documents.active_index(), Some(0));
}

#[test]
fn reopening_does_not_throw_away_unsaved_work() {
    let mut documents = open(&["a.rs", "b.rs"]);
    documents.activate(0);
    documents.active_mut().unwrap().insert("edited ");
    assert!(documents.active().unwrap().is_dirty());

    documents.open(doc("a.rs"));

    assert!(
        documents.active().unwrap().is_dirty(),
        "the open document was kept, not replaced by a fresh read"
    );
    assert!(documents.active().unwrap().text().starts_with("edited "));
}

#[test]
fn an_empty_set_has_no_active_document() {
    let documents = Documents::new();

    assert!(documents.is_empty());
    assert_eq!(documents.active_index(), None);
    assert!(documents.active().is_none());
}

mod closing {
    use super::*;

    #[test]
    fn closing_the_last_tab_leaves_nothing_open() {
        let mut documents = open(&["a.rs"]);

        documents.close(0);

        assert!(documents.is_empty());
        assert_eq!(documents.active_index(), None);
    }

    #[test]
    fn closing_the_active_tab_moves_focus_to_its_neighbour() {
        let mut documents = open(&["a.rs", "b.rs", "c.rs"]);
        documents.activate(1);

        documents.close(1);

        assert_eq!(documents.len(), 2);
        assert_eq!(
            documents.active().unwrap().path(),
            std::path::Path::new("c.rs"),
            "focus slides to what took its place"
        );
    }

    #[test]
    fn closing_the_last_one_in_the_row_falls_back_to_the_left() {
        let mut documents = open(&["a.rs", "b.rs"]);
        documents.activate(1);

        documents.close(1);

        assert_eq!(
            documents.active().unwrap().path(),
            std::path::Path::new("a.rs")
        );
    }

    #[test]
    fn closing_a_tab_before_the_active_one_keeps_the_same_document_focused() {
        let mut documents = open(&["a.rs", "b.rs", "c.rs"]);
        documents.activate(2);

        documents.close(0);

        assert_eq!(
            documents.active().unwrap().path(),
            std::path::Path::new("c.rs"),
            "the index shifted but the document did not"
        );
    }

    #[test]
    fn closing_a_tab_after_the_active_one_leaves_focus_alone() {
        let mut documents = open(&["a.rs", "b.rs", "c.rs"]);
        documents.activate(0);

        documents.close(2);

        assert_eq!(
            documents.active().unwrap().path(),
            std::path::Path::new("a.rs")
        );
    }

    #[test]
    fn closing_out_of_range_does_nothing() {
        let mut documents = open(&["a.rs"]);

        assert!(documents.close(9).is_none());
        assert_eq!(documents.len(), 1);
    }

    #[test]
    fn the_closed_document_comes_back_so_it_can_be_saved() {
        let mut documents = open(&["a.rs"]);
        documents.active_mut().unwrap().insert("x");

        let closed = documents.close_active().expect("the document");

        assert!(closed.is_dirty(), "the caller can still write it out");
    }
}

#[test]
fn each_tab_keeps_its_own_cursor() {
    let mut documents = open(&["a.rs", "b.rs"]);

    documents.activate(0);
    documents
        .active_mut()
        .unwrap()
        .move_cursor(Motion::DocumentEnd, false);
    let first = documents.active().unwrap().selection();

    documents.activate(1);
    assert_eq!(
        documents.active().unwrap().selection().head,
        0,
        "the other tab starts where it was left"
    );

    documents.activate(0);
    assert_eq!(documents.active().unwrap().selection(), first);
}

#[test]
fn the_dirty_ones_can_be_listed_for_saving() {
    let mut documents = open(&["a.rs", "b.rs", "c.rs"]);
    documents.activate(0);
    documents.active_mut().unwrap().insert("x");
    documents.activate(2);
    documents.active_mut().unwrap().insert("y");

    let dirty: Vec<_> = documents
        .dirty_paths()
        .into_iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect();

    assert_eq!(dirty, vec!["a.rs", "c.rs"]);
}

#[test]
fn activating_the_current_tab_is_not_a_change() {
    let mut documents = open(&["a.rs", "b.rs"]);

    assert!(!documents.activate(1), "already active");
    assert!(!documents.activate(9), "out of range");
    assert!(documents.activate(0));
}
