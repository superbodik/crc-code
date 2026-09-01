use serde::{Deserialize, Serialize};

/// A position as a human reads it: zero-based line, zero-based column.
///
/// Columns count characters, not bytes, so a column never lands inside a
/// multi-byte character. Offsets used elsewhere in this crate are character
/// offsets from the start of the buffer, for the same reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Point {
    pub line: usize,
    pub column: usize,
}

impl Point {
    pub const ZERO: Point = Point { line: 0, column: 0 };

    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

impl From<(usize, usize)> for Point {
    fn from((line, column): (usize, usize)) -> Self {
        Self::new(line, column)
    }
}
