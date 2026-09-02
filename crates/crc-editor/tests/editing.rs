use crc_editor::{Document, Motion};
use crc_text::{Point, Selection};
use crc_theme::Highlight;

fn document(text: &str) -> Document {
    Document::open("src/main.rs", text.to_string())
}

fn plain(text: &str) -> Document {
    Document::open("notes.txt", text.to_string())
}

#[test]
fn typing_lands_at_the_cursor_and_moves_it_along() {
    let mut doc = plain("hello world");
    doc.move_cursor(Motion::To(5), false);

    doc.insert(",");

    assert_eq!(doc.text(), "hello, world");
    assert_eq!(doc.selection(), Selection::cursor(6));
}

#[test]
fn typing_replaces_the_selection() {
    let mut doc = plain("hello world");
    doc.move_cursor(Motion::To(0), false);
    doc.move_cursor(Motion::To(5), true);

    doc.insert("goodbye");

    assert_eq!(doc.text(), "goodbye world");
    assert_eq!(doc.selection(), Selection::cursor(7));
}

#[test]
fn backspace_takes_the_character_behind_the_cursor() {
    let mut doc = plain("abc");
    doc.move_cursor(Motion::To(2), false);

    doc.backspace();

    assert_eq!(doc.text(), "ac");
    assert_eq!(doc.selection(), Selection::cursor(1));
}

#[test]
fn backspace_takes_the_selection_when_there_is_one() {
    let mut doc = plain("abcdef");
    doc.move_cursor(Motion::To(1), false);
    doc.move_cursor(Motion::To(4), true);

    doc.backspace();

    assert_eq!(doc.text(), "aef");
    assert_eq!(doc.selection(), Selection::cursor(1));
}

#[test]
fn backspace_at_the_start_does_nothing() {
    let mut doc = plain("abc");
    doc.backspace();

    assert_eq!(doc.text(), "abc");
    assert!(!doc.is_dirty());
}

#[test]
fn delete_takes_the_character_ahead() {
    let mut doc = plain("abc");
    doc.move_cursor(Motion::To(1), false);

    doc.delete();

    assert_eq!(doc.text(), "ac");
    assert_eq!(doc.selection(), Selection::cursor(1));
}

#[test]
fn delete_at_the_end_does_nothing() {
    let mut doc = plain("abc");
    doc.move_cursor(Motion::DocumentEnd, false);

    doc.delete();

    assert_eq!(doc.text(), "abc");
}

#[test]
fn a_newline_splits_the_line() {
    let mut doc = plain("abcd");
    doc.move_cursor(Motion::To(2), false);

    doc.insert("\n");

    assert_eq!(doc.text(), "ab\ncd");
    assert_eq!(doc.cursor(), Point::new(1, 0));
}

#[test]
fn multi_byte_text_counts_in_characters() {
    let mut doc = plain("привет");
    doc.move_cursor(Motion::To(6), false);

    doc.insert(" мир");

    assert_eq!(doc.text(), "привет мир");
    assert_eq!(doc.selection(), Selection::cursor(10));
    assert_eq!(doc.cursor_bytes(), "привет мир".len());
}

mod moving {
    use super::*;

    fn grid() -> Document {
        plain("alpha\nbe\ngamma line\n")
    }

    #[test]
    fn left_and_right_step_one_character() {
        let mut doc = plain("abc");
        doc.move_cursor(Motion::To(1), false);

        doc.move_cursor(Motion::Right, false);
        assert_eq!(doc.selection().head, 2);

        doc.move_cursor(Motion::Left, false);
        assert_eq!(doc.selection().head, 1);
    }

    #[test]
    fn the_ends_of_the_document_hold() {
        let mut doc = plain("abc");
        doc.move_cursor(Motion::Left, false);
        assert_eq!(doc.selection().head, 0);

        doc.move_cursor(Motion::DocumentEnd, false);
        doc.move_cursor(Motion::Right, false);
        assert_eq!(doc.selection().head, 3);
    }

    #[test]
    fn left_collapses_a_selection_to_its_start() {
        let mut doc = plain("abcdef");
        doc.move_cursor(Motion::To(1), false);
        doc.move_cursor(Motion::To(4), true);

        doc.move_cursor(Motion::Left, false);

        assert_eq!(doc.selection(), Selection::cursor(1));
    }

    #[test]
    fn right_collapses_a_selection_to_its_end() {
        let mut doc = plain("abcdef");
        doc.move_cursor(Motion::To(1), false);
        doc.move_cursor(Motion::To(4), true);

        doc.move_cursor(Motion::Right, false);

        assert_eq!(doc.selection(), Selection::cursor(4));
    }

    #[test]
    fn shift_extends_instead_of_collapsing() {
        let mut doc = plain("abcdef");
        doc.move_cursor(Motion::To(2), false);
        doc.move_cursor(Motion::Right, true);
        doc.move_cursor(Motion::Right, true);

        assert_eq!(doc.selection(), Selection::new(2, 4));
    }

    #[test]
    fn a_short_line_does_not_lose_the_column() {
        let mut doc = grid();
        doc.move_cursor(Motion::To(0), false);
        doc.move_cursor(Motion::LineEnd, false);
        assert_eq!(doc.cursor(), Point::new(0, 5));

        doc.move_cursor(Motion::Down, false);
        assert_eq!(doc.cursor(), Point::new(1, 2), "clamped to the short line");

        doc.move_cursor(Motion::Down, false);
        assert_eq!(
            doc.cursor(),
            Point::new(2, 5),
            "the column comes back on the longer line"
        );
    }

    #[test]
    fn moving_sideways_forgets_the_goal_column() {
        let mut doc = grid();
        doc.move_cursor(Motion::To(5), false);
        doc.move_cursor(Motion::Down, false);
        doc.move_cursor(Motion::Left, false);
        doc.move_cursor(Motion::Down, false);

        assert_eq!(
            doc.cursor(),
            Point::new(2, 1),
            "after a sideways step the new column is the goal"
        );
    }

    #[test]
    fn home_and_end_stay_on_the_line() {
        let mut doc = grid();
        doc.move_cursor(Motion::To(8), false);

        doc.move_cursor(Motion::LineStart, false);
        assert_eq!(doc.cursor(), Point::new(1, 0));

        doc.move_cursor(Motion::LineEnd, false);
        assert_eq!(doc.cursor(), Point::new(1, 2));
    }

    #[test]
    fn a_page_moves_by_the_rows_it_is_given() {
        let mut doc = plain("0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n");
        doc.move_cursor(Motion::PageDown(4), false);
        assert_eq!(doc.cursor().line, 4);

        doc.move_cursor(Motion::PageUp(2), false);
        assert_eq!(doc.cursor().line, 2);
    }

    #[test]
    fn a_page_past_the_end_stops_at_the_last_line() {
        let mut doc = plain("0\n1\n2\n");
        doc.move_cursor(Motion::PageDown(999), false);
        assert_eq!(doc.cursor().line, doc.line_count() - 1);
    }

    #[test]
    fn word_motion_steps_over_whole_words() {
        let mut doc = plain("let value = compute(x)");

        doc.move_cursor(Motion::WordRight, false);
        assert_eq!(doc.selection().head, 3, "past let");

        doc.move_cursor(Motion::WordRight, false);
        assert_eq!(doc.selection().head, 9, "past value");
    }

    #[test]
    fn word_motion_treats_punctuation_as_its_own_run() {
        let mut doc = plain("a == b");
        doc.move_cursor(Motion::WordRight, false);
        doc.move_cursor(Motion::WordRight, false);

        assert_eq!(doc.selection().head, 4, "the == is one step");
    }

    #[test]
    fn word_motion_backwards_lands_on_the_start_of_a_word() {
        let mut doc = plain("let value = 1");
        doc.move_cursor(Motion::DocumentEnd, false);

        doc.move_cursor(Motion::WordLeft, false);
        assert_eq!(doc.selection().head, 12);

        doc.move_cursor(Motion::WordLeft, false);
        assert_eq!(doc.selection().head, 10);
    }

    #[test]
    fn clicking_maps_a_point_to_an_offset() {
        let doc = grid();

        assert_eq!(doc.offset_at(Point::new(1, 1)), 7);
        assert_eq!(
            doc.offset_at(Point::new(1, 99)),
            8,
            "a click past the end of a line lands at its end"
        );
        assert_eq!(
            doc.offset_at(Point::new(99, 0)),
            doc.offset_at(Point::new(doc.line_count() - 1, 0)),
            "a click below the last line lands on it"
        );
    }
}

mod syntax {
    use super::*;

    fn role_at(doc: &Document, needle: &str) -> Option<Highlight> {
        let at = doc.text().find(needle)?;
        doc.highlights()
            .into_iter()
            .find(|(range, _)| range.start <= at && at < range.end)
            .map(|(_, highlight)| highlight)
    }

    #[test]
    fn a_file_is_highlighted_when_it_opens() {
        let doc = document("fn main() {}\n");
        assert_eq!(role_at(&doc, "fn"), Some(Highlight::Keyword));
    }

    #[test]
    fn highlighting_keeps_up_with_typing() {
        let mut doc = document("fn main() {}\n");
        doc.move_cursor(Motion::To(11), false);

        doc.insert("\n    let x = \"hi\";\n");

        assert_eq!(role_at(&doc, "let"), Some(Highlight::Keyword));
        assert_eq!(role_at(&doc, "\"hi\""), Some(Highlight::String));
    }

    #[test]
    fn deleting_re_highlights_too() {
        let mut doc = document("let x = \"hi\";\n");
        doc.move_cursor(Motion::To(8), false);
        doc.move_cursor(Motion::To(12), true);

        doc.backspace();

        assert_eq!(doc.text(), "let x = ;\n");
        assert_eq!(role_at(&doc, "\"hi\""), None, "the string is gone");
    }

    #[test]
    fn a_file_with_no_grammar_still_edits() {
        let mut doc = plain("plain text");
        doc.move_cursor(Motion::DocumentEnd, false);
        doc.insert("!");

        assert_eq!(doc.text(), "plain text!");
        assert!(doc.highlights().is_empty());
        assert_eq!(doc.language(), None);
    }

    #[test]
    fn undo_leaves_the_highlighting_correct() {
        let mut doc = document("fn main() {}\n");
        doc.move_cursor(Motion::To(0), false);
        doc.insert("pub ");
        doc.commit();
        assert_eq!(doc.text(), "pub fn main() {}\n");

        doc.undo();

        assert_eq!(doc.text(), "fn main() {}\n");
        assert_eq!(
            role_at(&doc, "fn"),
            Some(Highlight::Keyword),
            "the tree was rebuilt rather than reused against a stale edit"
        );
        assert_eq!(role_at(&doc, "main"), Some(Highlight::Function));
    }
}

mod saving {
    use super::*;

    #[test]
    fn a_fresh_document_is_clean() {
        assert!(!plain("hello").is_dirty());
    }

    #[test]
    fn an_edit_makes_it_dirty_and_saving_makes_it_clean() {
        let mut doc = plain("hello");
        doc.insert("!");
        assert!(doc.is_dirty());

        doc.mark_saved();
        assert!(!doc.is_dirty());
    }

    #[test]
    fn moving_the_cursor_does_not_make_it_dirty() {
        let mut doc = plain("hello");
        doc.move_cursor(Motion::DocumentEnd, false);
        doc.move_cursor(Motion::WordLeft, true);

        assert!(!doc.is_dirty());
    }

    #[test]
    fn undoing_back_to_the_saved_text_still_reads_as_dirty() {
        let mut doc = plain("hello");
        doc.insert("!");
        doc.commit();
        doc.undo();

        assert_eq!(doc.text(), "hello");
        assert!(
            doc.is_dirty(),
            "the version moved on, so the file is written again rather than assumed unchanged"
        );
    }
}

mod selecting {
    use super::*;

    #[test]
    fn select_all_covers_the_document() {
        let mut doc = plain("hello world");
        doc.select_all();

        assert_eq!(doc.selection(), Selection::new(0, 11));
        assert_eq!(doc.selected_bytes(), Some(0..11));
    }

    #[test]
    fn a_bare_cursor_selects_nothing() {
        let doc = plain("hello");
        assert_eq!(doc.selected_bytes(), None);
    }

    #[test]
    fn selected_bytes_are_bytes_not_characters() {
        let mut doc = plain("привет");
        doc.move_cursor(Motion::To(0), false);
        doc.move_cursor(Motion::To(3), true);

        assert_eq!(doc.selection().range(), 0..3);
        assert_eq!(doc.selected_bytes(), Some(0..6), "three two-byte letters");
    }

    #[test]
    fn typing_over_everything_replaces_the_document() {
        let mut doc = plain("throw this away");
        doc.select_all();
        doc.insert("new");

        assert_eq!(doc.text(), "new");
    }
}
