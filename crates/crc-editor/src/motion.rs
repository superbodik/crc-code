#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    Left,
    Right,
    Up,
    Down,
    WordLeft,
    WordRight,
    LineStart,
    LineEnd,
    DocumentStart,
    DocumentEnd,
    PageUp(usize),
    PageDown(usize),
    To(usize),
}

impl Motion {
    pub const fn keeps_goal_column(self) -> bool {
        matches!(
            self,
            Motion::Up | Motion::Down | Motion::PageUp(_) | Motion::PageDown(_)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharClass {
    Whitespace,
    Word,
    Punctuation,
}

pub fn class_of(character: char) -> CharClass {
    if character.is_whitespace() {
        CharClass::Whitespace
    } else if character.is_alphanumeric() || character == '_' {
        CharClass::Word
    } else {
        CharClass::Punctuation
    }
}
