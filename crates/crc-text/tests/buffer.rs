use crc_text::{Buffer, Edit, Point, Selection};

#[test]
fn inserts_deletes_and_replaces() {
    let mut buffer = Buffer::from_text("hello world");

    buffer.edit([Edit::insert(5, ",")]);
    assert_eq!(buffer.text(), "hello, world");

    buffer.edit([Edit::delete(5..6)]);
    assert_eq!(buffer.text(), "hello world");

    buffer.edit([Edit::replace(6..11, "there")]);
    assert_eq!(buffer.text(), "hello there");
}

#[test]
fn bumps_the_version_on_every_mutation() {
    let mut buffer = Buffer::from_text("a");
    assert_eq!(buffer.version(), 1);

    assert_eq!(buffer.edit([Edit::insert(1, "b")]), 2);
    assert_eq!(buffer.edit([Edit::insert(2, "c")]), 3);
    assert_eq!(buffer.undo(), Some(4));
    assert_eq!(buffer.version(), 4);
}

#[test]
fn applies_several_edits_against_the_same_snapshot() {
    let mut buffer = Buffer::from_text("one two three");

    buffer.edit([Edit::replace(0..3, "1"), Edit::replace(8..13, "3")]);

    assert_eq!(buffer.text(), "1 two 3");
}

#[test]
fn a_multi_edit_undoes_as_one_step() {
    let mut buffer = Buffer::from_text("one two three");
    buffer.edit([Edit::replace(0..3, "1"), Edit::replace(8..13, "3")]);

    buffer.undo();

    assert_eq!(buffer.text(), "one two three");
    assert!(!buffer.can_undo());
}

#[test]
fn coalesces_typing_into_one_undo_step() {
    let mut buffer = Buffer::from_text("");
    for (index, ch) in "hello".chars().enumerate() {
        buffer.edit([Edit::insert(index, ch.to_string())]);
    }
    assert_eq!(buffer.text(), "hello");

    buffer.undo();

    assert_eq!(buffer.text(), "", "five keystrokes are one undo step");
}

#[test]
fn breaks_the_undo_group_on_a_newline() {
    let mut buffer = Buffer::from_text("");
    buffer.edit([Edit::insert(0, "ab")]);
    buffer.edit([Edit::insert(2, "\n")]);
    buffer.edit([Edit::insert(3, "cd")]);

    buffer.undo();
    assert_eq!(buffer.text(), "ab\n");

    buffer.undo();
    assert_eq!(buffer.text(), "");
}

#[test]
fn breaks_the_undo_group_when_the_cursor_jumps() {
    let mut buffer = Buffer::from_text("....");
    buffer.edit([Edit::insert(0, "a")]);
    buffer.edit([Edit::insert(4, "b")]);

    buffer.undo();
    assert_eq!(buffer.text(), "a....");
}

#[test]
fn redoes_what_was_undone() {
    let mut buffer = Buffer::from_text("start");
    buffer.edit([Edit::insert(5, "!")]);
    buffer.commit();

    buffer.undo();
    assert_eq!(buffer.text(), "start");
    assert!(buffer.can_redo());

    buffer.redo();
    assert_eq!(buffer.text(), "start!");
}

#[test]
fn a_new_edit_drops_the_redo_stack() {
    let mut buffer = Buffer::from_text("");
    buffer.edit([Edit::insert(0, "a")]);
    buffer.undo();
    assert!(buffer.can_redo());

    buffer.edit([Edit::insert(0, "b")]);

    assert!(!buffer.can_redo());
    assert_eq!(buffer.text(), "b");
}

#[test]
fn undoes_nothing_on_an_untouched_buffer() {
    let mut buffer = Buffer::from_text("x");
    assert_eq!(buffer.undo(), None);
    assert_eq!(buffer.text(), "x");
}

#[test]
fn converts_between_offsets_and_points() {
    let buffer = Buffer::from_text("one\ntwo\nthree");

    assert_eq!(buffer.offset_to_point(0), Point::new(0, 0));
    assert_eq!(buffer.offset_to_point(4), Point::new(1, 0));
    assert_eq!(buffer.offset_to_point(9), Point::new(2, 1));

    assert_eq!(buffer.point_to_offset(Point::new(1, 0)), 4);
    assert_eq!(buffer.point_to_offset(Point::new(2, 1)), 9);
}

#[test]
fn clamps_positions_that_fall_outside_the_buffer() {
    let buffer = Buffer::from_text("one\ntwo");

    assert_eq!(buffer.point_to_offset(Point::new(0, 99)), 3);
    assert_eq!(buffer.point_to_offset(Point::new(99, 0)), 4);
    assert_eq!(buffer.offset_to_point(999), Point::new(1, 3));
}

#[test]
fn handles_multi_byte_characters_by_character_not_byte() {
    let mut buffer = Buffer::from_text("привет мир");

    assert_eq!(buffer.len_chars(), 10);
    assert_eq!(buffer.point_to_offset(Point::new(0, 6)), 6);

    buffer.edit([Edit::replace(7..10, "світ")]);
    assert_eq!(buffer.text(), "привет світ");
}

#[test]
fn reads_lines_without_their_newline() {
    let buffer = Buffer::from_text("one\ntwo\r\nthree");

    assert_eq!(buffer.line(0).as_deref(), Some("one"));
    assert_eq!(buffer.line(1).as_deref(), Some("two"));
    assert_eq!(buffer.line(2).as_deref(), Some("three"));
    assert_eq!(buffer.line(3), None);
}

#[test]
fn clamps_an_edit_range_past_the_end() {
    let mut buffer = Buffer::from_text("abc");
    buffer.edit([Edit::replace(2..999, "Z")]);
    assert_eq!(buffer.text(), "abZ");
}

#[test]
fn a_selection_keeps_its_direction() {
    let backwards = Selection::new(10, 4);
    assert_eq!(backwards.range(), 4..10);
    assert_eq!(backwards.collapsed(), Selection::cursor(4));
    assert!(!backwards.is_empty());
    assert!(Selection::cursor(3).is_empty());
}

#[test]
fn a_selection_follows_an_edit_before_it() {
    let selection = Selection::new(10, 12);

    let moved = selection.shifted(&(0..2), 5);

    assert_eq!(moved, Selection::new(13, 15));
}

#[test]
fn a_selection_inside_a_replaced_span_collapses_to_its_start() {
    let selection = Selection::new(6, 8);

    let moved = selection.shifted(&(4..10), 1);

    assert_eq!(moved, Selection::new(4, 4));
}

#[test]
fn a_selection_before_an_edit_does_not_move() {
    let selection = Selection::new(1, 3);
    assert_eq!(selection.shifted(&(10..12), 40), selection);
}
