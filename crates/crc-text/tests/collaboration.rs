use crc_text::edit::rebase;
use crc_text::{AuthorId, Buffer, Change, Edit};

const PEER: AuthorId = AuthorId(2);

#[test]
fn undo_takes_back_my_edit_not_the_one_on_top() {
    let mut buffer = Buffer::from_text("aaa bbb");

    buffer.edit_as(AuthorId::LOCAL, [Edit::insert(0, "L")]);
    buffer.edit_as(PEER, [Edit::insert(8, "P")]);
    assert_eq!(buffer.text(), "Laaa bbbP");

    buffer.undo_by(AuthorId::LOCAL).expect("my edit undoes");

    assert_eq!(
        buffer.text(),
        "aaa bbbP",
        "my insert is gone and the peer's is untouched"
    );
}

#[test]
fn undo_shifts_past_edits_made_before_mine_in_the_file() {
    let mut buffer = Buffer::from_text("aaa bbb");

    buffer.edit_as(AuthorId::LOCAL, [Edit::insert(7, "L")]);
    buffer.edit_as(PEER, [Edit::insert(0, "PPP")]);
    assert_eq!(buffer.text(), "PPPaaa bbbL");

    buffer.undo_by(AuthorId::LOCAL).expect("my edit undoes");

    assert_eq!(buffer.text(), "PPPaaa bbb");
}

#[test]
fn undo_refuses_when_a_peer_edited_the_same_text() {
    let mut buffer = Buffer::from_text("hello");

    buffer.edit_as(AuthorId::LOCAL, [Edit::replace(0..5, "hey")]);
    buffer.edit_as(PEER, [Edit::replace(0..3, "yo")]);

    assert_eq!(
        buffer.undo_by(AuthorId::LOCAL),
        None,
        "a conflict must not resolve itself by discarding the peer's edit"
    );
    assert_eq!(buffer.text(), "yo");
}

#[test]
fn undo_finds_nothing_for_an_author_who_has_not_typed() {
    let mut buffer = Buffer::from_text("x");
    buffer.edit_as(PEER, [Edit::insert(1, "y")]);

    assert!(!buffer.can_undo_by(AuthorId::LOCAL));
    assert_eq!(buffer.undo_by(AuthorId::LOCAL), None);
    assert_eq!(buffer.text(), "xy");
}

#[test]
fn redo_after_a_collaborative_undo_puts_it_back_where_it_belongs() {
    let mut buffer = Buffer::from_text("aaa bbb");

    buffer.edit_as(AuthorId::LOCAL, [Edit::insert(0, "L")]);
    buffer.edit_as(PEER, [Edit::insert(8, "P")]);
    buffer.undo_by(AuthorId::LOCAL);
    assert_eq!(buffer.text(), "aaa bbbP");

    buffer.redo().expect("redo");

    assert_eq!(buffer.text(), "Laaa bbbP");
}

#[test]
fn typing_by_two_authors_does_not_coalesce_into_one_step() {
    let mut buffer = Buffer::from_text("");

    buffer.edit_as(AuthorId::LOCAL, [Edit::insert(0, "a")]);
    buffer.edit_as(PEER, [Edit::insert(1, "b")]);
    buffer.edit_as(AuthorId::LOCAL, [Edit::insert(2, "c")]);
    assert_eq!(buffer.text(), "abc");

    buffer.undo();
    assert_eq!(buffer.text(), "ab");
    buffer.undo();
    assert_eq!(buffer.text(), "a");
}

#[test]
fn an_agents_work_undoes_as_its_own_step() {
    let mut buffer = Buffer::from_text("one two three");

    buffer.edit_as(AuthorId::LOCAL, [Edit::insert(0, "// ")]);
    buffer.edit_as(
        AuthorId::AGENT,
        [Edit::replace(3..6, "1"), Edit::replace(11..16, "3")],
    );
    assert_eq!(buffer.text(), "// 1 two 3");

    buffer.undo_by(AuthorId::AGENT).expect("the agent's diff");

    assert_eq!(
        buffer.text(),
        "// one two three",
        "the whole diff comes back out, my comment stays"
    );
}

#[test]
fn an_author_id_is_local_only_for_this_keyboard() {
    assert!(AuthorId::LOCAL.is_local());
    assert!(!AuthorId::AGENT.is_local());
    assert!(!AuthorId::peer(0).is_local());
    assert_ne!(AuthorId::peer(0), AuthorId::peer(1));
}

mod rebasing {
    use super::*;

    fn change(range: std::ops::Range<usize>, removed: &str, inserted: &str) -> Change {
        Change {
            range,
            removed: removed.to_string(),
            inserted: inserted.to_string(),
        }
    }

    #[test]
    fn a_range_before_the_change_stays_put() {
        let over = change(10..10, "", "xyz");
        assert_eq!(rebase(&(2..5), &over), Some(2..5));
    }

    #[test]
    fn a_range_after_an_insertion_slides_forward() {
        let over = change(0..0, "", "xyz");
        assert_eq!(rebase(&(2..5), &over), Some(5..8));
    }

    #[test]
    fn a_range_after_a_deletion_slides_back() {
        let over = change(0..3, "abc", "");
        assert_eq!(rebase(&(5..7), &over), Some(2..4));
    }

    #[test]
    fn a_range_touching_the_change_is_a_conflict() {
        let over = change(4..8, "abcd", "z");
        assert_eq!(rebase(&(6..7), &over), None, "inside");
        assert_eq!(rebase(&(2..6), &over), None, "overlapping the start");
        assert_eq!(rebase(&(6..12), &over), None, "overlapping the end");
    }

    #[test]
    fn a_cursor_on_the_seam_of_an_insertion_is_not_a_conflict() {
        let over = change(5..5, "", "new");
        assert_eq!(rebase(&(5..5), &over), Some(5..5));
    }

    #[test]
    fn an_adjacent_range_is_not_a_conflict() {
        let over = change(4..8, "abcd", "z");
        assert_eq!(rebase(&(0..4), &over), Some(0..4), "ends where it begins");
        assert_eq!(rebase(&(8..10), &over), Some(5..7), "begins where it ends");
    }
}
