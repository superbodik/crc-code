use std::ops::Range;

use serde::{Deserialize, Serialize};

/// A cursor, and the text it has selected.
///
/// `anchor` is where the selection was started and `head` is where the cursor
/// is now, so a selection dragged backwards has `head < anchor`. Keeping both
/// is what lets shift-arrow extend a selection in either direction without
/// losing the origin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Selection {
    /// Character offset where the selection began.
    pub anchor: usize,
    /// Character offset where the cursor sits.
    pub head: usize,
}

impl Selection {
    /// A bare cursor with nothing selected.
    pub fn cursor(at: usize) -> Self {
        Self {
            anchor: at,
            head: at,
        }
    }

    pub fn new(anchor: usize, head: usize) -> Self {
        Self { anchor, head }
    }

    pub fn is_empty(&self) -> bool {
        self.anchor == self.head
    }

    pub fn start(&self) -> usize {
        self.anchor.min(self.head)
    }

    pub fn end(&self) -> usize {
        self.anchor.max(self.head)
    }

    /// The selected span, in ascending order regardless of drag direction.
    pub fn range(&self) -> Range<usize> {
        self.start()..self.end()
    }

    /// Collapse to a plain cursor at the head.
    pub fn collapsed(&self) -> Self {
        Self::cursor(self.head)
    }

    /// Where this selection lands after an edit elsewhere in the buffer.
    ///
    /// Text inserted before it pushes it along; text deleted around it pulls it
    /// back to the edit site. Without this, every cursor and every marker would
    /// have to be recomputed from scratch after each keystroke.
    pub fn shifted(&self, edit_range: &Range<usize>, inserted_len: usize) -> Self {
        Self {
            anchor: shift_offset(self.anchor, edit_range, inserted_len),
            head: shift_offset(self.head, edit_range, inserted_len),
        }
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self::cursor(0)
    }
}

fn shift_offset(offset: usize, edit: &Range<usize>, inserted_len: usize) -> usize {
    if offset <= edit.start {
        offset
    } else if offset >= edit.end {
        // Entirely after the edit: move by the net change in length.
        offset - (edit.end - edit.start) + inserted_len
    } else {
        // Inside the replaced span, which no longer exists: clamp to its start.
        edit.start
    }
}
