use crc_editor::{Document, Motion};

fn code(text: &str) -> Document {
    Document::open("src/main.rs", text.to_string())
}

#[test]
fn a_reload_replaces_the_text() {
    let mut doc = code("fn one() {}\n");
    doc.reload("fn two() {}\n".to_string());

    assert_eq!(doc.text(), "fn two() {}\n");
}

#[test]
fn a_reloaded_file_is_not_dirty() {
    let mut doc = code("one\n");
    doc.insert("x");
    assert!(doc.is_dirty());

    doc.reload("two\n".to_string());
    assert!(!doc.is_dirty(), "what came from disk is what is on disk");
}

#[test]
fn the_cursor_is_kept_where_it_still_fits() {
    let mut doc = code("one\ntwo\nthree\n");
    doc.move_cursor(Motion::To(5), false);

    doc.reload("one\ntwo\nthree\nfour\n".to_string());

    assert_eq!(doc.selection().head, 5);
}

#[test]
fn a_cursor_past_the_end_of_a_shorter_file_is_pulled_back() {
    let mut doc = code("one\ntwo\nthree\n");
    doc.move_cursor(Motion::DocumentEnd, false);

    doc.reload("hi\n".to_string());

    assert!(
        doc.selection().head <= doc.text().chars().count(),
        "the cursor cannot sit outside the file"
    );
}

#[test]
fn highlighting_follows_the_new_text() {
    let mut doc = code("fn one() {}\n");
    doc.reload("struct Two;\n".to_string());

    let highlights = doc.highlights();
    assert!(!highlights.is_empty(), "the new text was never parsed");
    assert!(
        highlights
            .iter()
            .any(|(range, _)| doc.text()[range.clone()].contains("struct")),
        "the keyword of the new text should be marked"
    );
}

#[test]
fn faults_are_recomputed_on_reload() {
    let mut doc = code("fn one() {}\n");
    assert!(doc.faults().is_empty());

    doc.reload("fn one( {\n".to_string());
    assert!(!doc.faults().is_empty(), "the new breakage should be seen");

    doc.reload("fn one() {}\n".to_string());
    assert!(doc.faults().is_empty(), "and the repair should clear it");
}

#[test]
fn a_selection_does_not_survive_into_a_file_it_no_longer_describes() {
    let mut doc = code("one\ntwo\nthree\n");
    doc.move_cursor(Motion::To(0), false);
    doc.move_cursor(Motion::To(7), true);
    assert!(doc.selected_text().is_some());

    doc.reload("completely different\n".to_string());

    assert_eq!(
        doc.selected_text(),
        None,
        "a range measured against the old text means nothing now"
    );
}
