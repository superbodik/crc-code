use std::ops::Range;

use ropey::Rope;

use crate::author::AuthorId;
use crate::edit::{Change, Edit};
use crate::history::{History, Transaction};
use crate::point::Point;

#[derive(Debug)]
pub struct Buffer {
    rope: Rope,
    version: u64,
    history: History,
}

impl Buffer {
    pub fn new() -> Self {
        Self::from_text("")
    }

    pub fn from_text(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
            version: 1,
            history: History::default(),
        }
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    pub fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn is_empty(&self) -> bool {
        self.rope.len_chars() == 0
    }

    pub fn line(&self, line: usize) -> Option<String> {
        if line >= self.rope.len_lines() {
            return None;
        }
        let slice = self.rope.line(line);
        Some(slice.to_string().trim_end_matches(['\n', '\r']).to_string())
    }

    pub fn slice(&self, range: Range<usize>) -> String {
        let range = self.clamp_range(range);
        self.rope.slice(range).to_string()
    }

    pub fn edit(&mut self, edits: impl IntoIterator<Item = Edit>) -> u64 {
        self.edit_as(AuthorId::LOCAL, edits)
    }

    pub fn edit_as(&mut self, author: AuthorId, edits: impl IntoIterator<Item = Edit>) -> u64 {
        let mut edits: Vec<Edit> = edits.into_iter().collect();
        if edits.is_empty() {
            return self.version;
        }
        edits.sort_by_key(|edit| std::cmp::Reverse(edit.range.start));

        if edits.len() == 1 {
            let edit = edits.pop().expect("one edit");
            let change = self.apply(edit.range, &edit.text);
            self.history.push(change, author);
        } else {
            let changes = edits
                .into_iter()
                .map(|edit| self.apply(edit.range, &edit.text))
                .collect();
            self.history
                .push_transaction(Transaction { author, changes });
        }

        self.version += 1;
        self.version
    }

    pub fn commit(&mut self) {
        self.history.commit();
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    pub fn undo(&mut self) -> Option<u64> {
        let transaction = self.history.undo()?;
        self.replay(&transaction);
        self.version += 1;
        Some(self.version)
    }

    pub fn undo_by(&mut self, author: AuthorId) -> Option<u64> {
        let transaction = self.history.undo_by(author)?;
        self.replay(&transaction);
        self.version += 1;
        Some(self.version)
    }

    pub fn can_undo_by(&self, author: AuthorId) -> bool {
        self.history.can_undo_by(author)
    }

    pub fn redo(&mut self) -> Option<u64> {
        let transaction = self.history.redo()?;
        self.replay(&transaction);
        self.version += 1;
        Some(self.version)
    }

    pub fn char_to_byte(&self, offset: usize) -> usize {
        self.rope.char_to_byte(offset.min(self.rope.len_chars()))
    }

    pub fn byte_to_char(&self, byte: usize) -> usize {
        let byte = byte.min(self.rope.len_bytes());
        let byte = (0..=byte)
            .rev()
            .find(|b| self.rope.try_byte_to_char(*b).is_ok());
        self.rope.byte_to_char(byte.unwrap_or(0))
    }

    pub fn len_bytes(&self) -> usize {
        self.rope.len_bytes()
    }

    pub fn line_start(&self, line: usize) -> usize {
        let line = line.min(self.rope.len_lines().saturating_sub(1));
        self.rope.line_to_char(line)
    }

    pub fn line_len(&self, line: usize) -> usize {
        self.line(line)
            .map(|text| text.chars().count())
            .unwrap_or(0)
    }

    pub fn point_to_offset(&self, point: Point) -> usize {
        let last_line = self.rope.len_lines().saturating_sub(1);
        let line = point.line.min(last_line);
        let line_start = self.rope.line_to_char(line);
        let line_len = self
            .line(line)
            .map(|text| text.chars().count())
            .unwrap_or(0);
        line_start + point.column.min(line_len)
    }

    pub fn offset_to_point(&self, offset: usize) -> Point {
        let offset = offset.min(self.rope.len_chars());
        let line = self.rope.char_to_line(offset);
        Point {
            line,
            column: offset - self.rope.line_to_char(line),
        }
    }

    fn apply(&mut self, range: Range<usize>, text: &str) -> Change {
        let range = self.clamp_range(range);
        let removed = self.rope.slice(range.clone()).to_string();

        if !range.is_empty() {
            self.rope.remove(range.clone());
        }
        if !text.is_empty() {
            self.rope.insert(range.start, text);
        }

        Change {
            range,
            removed,
            inserted: text.to_string(),
        }
    }

    fn replay(&mut self, transaction: &Transaction) {
        for change in &transaction.changes {
            self.apply(change.range.clone(), &change.inserted);
        }
    }

    fn clamp_range(&self, range: Range<usize>) -> Range<usize> {
        let len = self.rope.len_chars();
        let start = range.start.min(len);
        let end = range.end.min(len).max(start);
        start..end
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&str> for Buffer {
    fn from(text: &str) -> Self {
        Buffer::from_text(text)
    }
}
