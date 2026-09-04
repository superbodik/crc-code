use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fault {
    pub range: Range<usize>,
    pub line: usize,
    pub column: usize,
    pub missing: bool,
    pub kind: String,
}

impl Fault {
    pub fn message(&self) -> String {
        if self.missing {
            format!("не хватает {}", self.kind)
        } else {
            "разбор здесь ломается".to_string()
        }
    }
}
