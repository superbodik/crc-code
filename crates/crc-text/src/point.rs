use serde::{Deserialize, Serialize};

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
