use std::ops::Range;

use ropey::Rope;

use crate::edit::{Change, Edit};
use crate::history::{History, Transaction};
use crate::point::Point;

/// An editable text buffer.
///
/// Text lives in a rope, so an edit in the middle of a large file costs about
/// the same as one at the start — inserting a character into a 100k-line file
/// does not copy the file. Every offset in this API is a character offset;
/// [`Point`] converts to and from line/column.
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

    /// Bumped on every mutation, including undo and redo.
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

    /// One line, without its trailing newline. Out-of-range lines give `None`.
    pub fn line(&self, line: usize) -> Option<String> {
        if line >= self.rope.len_lines() {
            return None;
        }
        let slice = self.rope.line(line);
        Some(slice.to_string().trim_end_matches(['\n', '\r']).to_string())
    }

    /// The text in a character range, clamped to the buffer.
    pub fn slice(&self, range: Range<usize>) -> String {
        let range = self.clamp_range(range);
        self.rope.slice(range).to_string()
    }

    /// Apply one or more edits as a single step.
    ///
    /// Edits are given against the *current* buffer — they are applied back to
    /// front so that the offsets in each one stay valid, which is what makes
    /// multi-cursor editing work without the caller rebasing anything.
    pub fn edit(&mut self, edits: impl IntoIterator<Item = Edit>) -> u64 {
        let mut edits: Vec<Edit> = edits.into_iter().collect();
        if edits.is_empty() {
            return self.version;
        }
        edits.sort_by_key(|edit| std::cmp::Reverse(edit.range.start));

        if edits.len() == 1 {
            let edit = edits.pop().expect("one edit");
            let change = self.apply(edit.range, &edit.text);
            // Single edits feed the coalescing path, so typing undoes by word.
            self.history.push(change);
        } else {
            let changes = edits
                .into_iter()
                .map(|edit| self.apply(edit.range, &edit.text))
                .collect();
            self.history.push_transaction(Transaction { changes });
        }

        self.version += 1;
        self.version
    }

    /// Close the current undo group. Call on cursor jumps, on save, or after an
    /// idle pause.
    pub fn commit(&mut self) {
        self.history.commit();
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Step back one group. `None` when there is nothing to undo.
    pub fn undo(&mut self) -> Option<u64> {
        let transaction = self.history.undo()?;
        self.replay(&transaction);
        self.version += 1;
        Some(self.version)
    }

    /// Step forward one group. `None` when there is nothing to redo.
    pub fn redo(&mut self) -> Option<u64> {
        let transaction = self.history.redo()?;
        self.replay(&transaction);
        self.version += 1;
        Some(self.version)
    }

    /// Line/column to character offset. Out-of-range input clamps to the end of
    /// the buffer rather than panicking — these values come from plugins and
    /// agents, not just from the editor itself.
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

    /// Character offset to line/column.
    pub fn offset_to_point(&self, offset: usize) -> Point {
        let offset = offset.min(self.rope.len_chars());
        let line = self.rope.char_to_line(offset);
        Point {
            line,
            column: offset - self.rope.line_to_char(line),
        }
    }

    /// Replace a range, returning the change that did it.
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

    /// Apply a transaction without recording it — used by undo and redo, which
    /// manage the stacks themselves.
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
