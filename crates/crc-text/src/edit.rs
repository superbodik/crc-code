use std::ops::Range;

use serde::{Deserialize, Serialize};

/// Replace a character range with new text.
///
/// One shape covers all three operations: insert is an empty range, delete is
/// empty text, replace is neither. Everything that mutates a buffer goes
/// through this, so history and change notification have a single case to
/// handle rather than three.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Edit {
    /// Character offsets into the buffer.
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

/// An edit paired with the text it displaced — enough to undo it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    pub range: Range<usize>,
    /// What was there before.
    pub removed: String,
    /// What is there now.
    pub inserted: String,
}

impl Change {
    /// The range this change occupies *after* it was applied.
    pub fn applied_range(&self) -> Range<usize> {
        self.range.start..self.range.start + self.inserted.chars().count()
    }

    /// The change that puts the buffer back the way it was.
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

    /// How many characters this change added, minus what it took away.
    pub fn delta(&self) -> isize {
        self.inserted.chars().count() as isize - self.removed.chars().count() as isize
    }
}

/// Move a range from the coordinates before `over` was applied into the
/// coordinates after it.
///
/// `None` when the two overlap: the text the range referred to is not there
/// any more, so no honest answer exists. Callers treat that as a conflict
/// rather than guessing — which is what keeps a collaborative undo from
/// silently eating someone else's edit.
pub fn rebase(range: &Range<usize>, over: &Change) -> Option<Range<usize>> {
    let removed_len = over.removed.chars().count();
    let over_end = over.range.start + removed_len;

    if range.end <= over.range.start {
        // Entirely before the change.
        return Some(range.clone());
    }
    if range.start >= over_end {
        // Entirely after it: slide by the net length change.
        let shift = over.delta();
        let start = range.start.checked_add_signed(shift)?;
        let end = range.end.checked_add_signed(shift)?;
        return Some(start..end);
    }
    // An empty range sitting exactly on the boundary of an insertion is not a
    // real overlap — it is a cursor, and it stays where it is.
    if removed_len == 0 && range.is_empty() {
        return Some(range.clone());
    }
    None
}
