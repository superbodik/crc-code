use crc_editor::{Document, Motion};

fn plain(text: &str) -> Document {
    Document::open("notes.txt", text.to_string())
}

#[test]
fn every_occurrence_is_found_in_order() {
    let doc = plain("one two one two one");
    let found = doc.find("one", false);

    assert_eq!(found.len(), 3);
    assert_eq!(found[0], 0..3);
    assert_eq!(found[1], 8..11);
    assert_eq!(found[2], 16..19);
}

#[test]
fn an_empty_query_finds_nothing() {
    assert!(plain("anything at all").find("", false).is_empty());
}

#[test]
fn case_is_ignored_unless_it_is_asked_for() {
    let doc = plain("Rect rect RECT");

    assert_eq!(doc.find("rect", false).len(), 3);
    assert_eq!(doc.find("rect", true).len(), 1);
    assert_eq!(doc.find("Rect", true).len(), 1);
}

#[test]
fn a_query_that_is_not_there_finds_nothing() {
    assert!(plain("one two three").find("four", false).is_empty());
}

#[test]
fn overlapping_runs_are_all_reported() {
    let found = plain("aaaa").find("aa", false);

    assert_eq!(found.len(), 3, "aa sits at 0, 1 and 2");
}

#[test]
fn a_match_can_cross_a_line_break() {
    let doc = plain("first\nsecond");
    let found = doc.find("t\ns", false);

    assert_eq!(found.len(), 1);
    assert_eq!(found[0], 4..7);
}

#[test]
fn offsets_are_counted_in_characters_not_bytes() {
    let doc = plain("привет мир привет");
    let found = doc.find("привет", false);

    assert_eq!(found.len(), 2);
    assert_eq!(found[0], 0..6);
    assert_eq!(found[1], 11..17, "the second one starts at character 11");
}

#[test]
fn a_cyrillic_query_is_matched_without_regard_to_case() {
    let doc = plain("Привет привет ПРИВЕТ");
    assert_eq!(doc.find("привет", false).len(), 3);
    assert_eq!(doc.find("привет", true).len(), 1);
}

#[test]
fn a_found_range_can_be_selected_and_read_back() {
    let mut doc = plain("one two three");
    let found = doc.find("two", false);

    doc.select_range(found[0].clone());

    assert_eq!(doc.selected_text().as_deref(), Some("two"));
}

#[test]
fn ranges_convert_between_characters_and_bytes() {
    let mut doc = plain("привет мир");
    let found = doc.find("мир", false);
    let bytes = doc.byte_range(found[0].clone());

    assert_eq!(bytes, 13..19, "each Cyrillic letter is two bytes");
    assert_eq!(doc.char_range(bytes), found[0]);

    doc.select_range(doc.char_range(13..19));
    assert_eq!(doc.selected_text().as_deref(), Some("мир"));
}

#[test]
fn editing_the_file_moves_the_matches() {
    let mut doc = plain("one two one");
    assert_eq!(doc.find("one", false).len(), 2);

    doc.move_cursor(Motion::To(0), false);
    doc.insert("one ");

    assert_eq!(doc.find("one", false).len(), 3);
    assert_eq!(doc.find("one", false)[0], 0..3);
}
