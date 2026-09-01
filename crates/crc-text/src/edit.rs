use std::ops::Range;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Edit {
    pub range: Range<usize>,
    pub text: String,
}

impl Edit {
    pub fn insert(at: usize, text: impl Into<String>) -> Self {
        Self {
            range: at..at,
            text: text.into(),
        }
    }

    pub fn delete(range: Range<usize>) -> Self {
        Self {
            range,
            text: String::new(),
        }
    }

    pub fn replace(range: Range<usize>, text: impl Into<String>) -> Self {
        Self {
            range,
            text: text.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    pub range: Range<usize>,
    pub removed: String,
    pub inserted: String,
}

impl Change {
    pub fn applied_range(&self) -> Range<usize> {
        self.range.start..self.range.start + self.inserted.chars().count()
    }

    pub fn inverted(&self) -> Change {
        Change {
            range: self.applied_range(),
            removed: self.inserted.clone(),
            inserted: self.removed.clone(),
        }
    }

    pub fn as_edit(&self) -> Edit {
        Edit {
            range: self.range.clone(),
            text: self.inserted.clone(),
        }
    }

    pub fn delta(&self) -> isize {
        self.inserted.chars().count() as isize - self.removed.chars().count() as isize
    }
}

pub fn rebase(range: &Range<usize>, over: &Change) -> Option<Range<usize>> {
    let removed_len = over.removed.chars().count();
    let over_end = over.range.start + removed_len;

    if range.end <= over.range.start {
        return Some(range.clone());
    }
    if range.start >= over_end {
        let shift = over.delta();
        let start = range.start.checked_add_signed(shift)?;
        let end = range.end.checked_add_signed(shift)?;
        return Some(start..end);
    }
    if removed_len == 0 && range.is_empty() {
        return Some(range.clone());
    }
    None
}
