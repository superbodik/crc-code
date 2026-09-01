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
}
