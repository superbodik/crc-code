use std::ops::Range;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Selection {
    pub anchor: usize,
    pub head: usize,
}

impl Selection {
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

    pub fn range(&self) -> Range<usize> {
        self.start()..self.end()
    }

    pub fn collapsed(&self) -> Self {
        Self::cursor(self.head)
    }

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
        offset - (edit.end - edit.start) + inserted_len
    } else {
        edit.start
    }
}
