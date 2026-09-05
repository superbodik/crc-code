use std::ops::Range;
use std::path::{Path, PathBuf};

use crc_syntax::{Language, SyntaxTree};
use crc_text::{Buffer, Edit, Point, Selection};
use crc_theme::Highlight;

use crate::motion::{CharClass, Motion, class_of};

pub struct Document {
    path: PathBuf,
    buffer: Buffer,
    text: String,
    tree: Option<SyntaxTree>,
    language: Option<Language>,
    selection: Selection,
    goal_column: Option<usize>,
    saved_version: u64,
}

impl Document {
    pub fn open(path: impl Into<PathBuf>, text: String) -> Self {
        let path = path.into();
        let language = Language::from_path(&path);

        let tree = language.and_then(|language| {
            let mut tree = SyntaxTree::new(language).ok()?;
            tree.parse(&text).ok()?;
            Some(tree)
        });

        let buffer = Buffer::from_text(&text);
        let saved_version = buffer.version();

        Self {
            path,
            buffer,
            text,
            tree,
            language,
            selection: Selection::cursor(0),
            goal_column: None,
            saved_version,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn language(&self) -> Option<Language> {
        self.language
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn selection(&self) -> Selection {
        self.selection
    }

    pub fn cursor(&self) -> Point {
        self.buffer.offset_to_point(self.selection.head)
    }

    pub fn line_count(&self) -> usize {
        self.buffer.len_lines()
    }

    pub fn is_dirty(&self) -> bool {
        self.buffer.version() != self.saved_version
    }

    pub fn reload(&mut self, text: String) {
        let cursor = self.selection.head.min(text.chars().count());

        self.buffer = Buffer::from_text(&text);
        self.text = text;
        self.selection = Selection::cursor(cursor);
        self.goal_column = None;
        self.saved_version = self.buffer.version();

        if let Some(tree) = self.tree.as_mut() {
            tree.reset();
            let _ = tree.parse(&self.text);
        }
    }

    pub fn mark_saved(&mut self) {
        self.saved_version = self.buffer.version();
        self.buffer.commit();
    }

    pub fn faults(&self) -> Vec<crc_syntax::Fault> {
        match self.tree.as_ref() {
            Some(tree) => tree.faults(),
            None => Vec::new(),
        }
    }

    pub fn highlights(&self) -> Vec<(Range<usize>, Highlight)> {
        match self.tree.as_ref() {
            Some(tree) => tree
                .highlights(&self.text)
                .into_iter()
                .map(|span| (span.range, span.highlight))
                .collect(),
            None => Vec::new(),
        }
    }

    pub fn selected_bytes(&self) -> Option<Range<usize>> {
        if self.selection.is_empty() {
            return None;
        }
        let range = self.selection.range();
        Some(self.buffer.char_to_byte(range.start)..self.buffer.char_to_byte(range.end))
    }

    pub fn cursor_bytes(&self) -> usize {
        self.buffer.char_to_byte(self.selection.head)
    }

    pub fn insert(&mut self, text: &str) {
        let range = self.selection.range();
        self.replace(range.clone(), text);
        self.place(range.start + text.chars().count(), false);
    }

    pub fn backspace(&mut self) {
        if !self.selection.is_empty() {
            let range = self.selection.range();
            self.replace(range.clone(), "");
            self.place(range.start, false);
            return;
        }
        let head = self.selection.head;
        if head == 0 {
            return;
        }
        self.replace(head - 1..head, "");
        self.place(head - 1, false);
    }

    pub fn delete(&mut self) {
        if !self.selection.is_empty() {
            let range = self.selection.range();
            self.replace(range.clone(), "");
            self.place(range.start, false);
            return;
        }
        let head = self.selection.head;
        if head >= self.buffer.len_chars() {
            return;
        }
        self.replace(head..head + 1, "");
        self.place(head, false);
    }

    pub fn delete_word(&mut self, forward: bool) {
        if !self.selection.is_empty() {
            self.backspace();
            return;
        }

        let head = self.selection.head;
        let target = self.word_boundary(head, if forward { 1 } else { -1 });
        if target == head {
            return;
        }

        let (start, end) = if forward {
            (head, target)
        } else {
            (target, head)
        };
        self.replace(start..end, "");
        self.place(start, false);
    }

    pub fn find(&self, needle: &str, case_sensitive: bool) -> Vec<Range<usize>> {
        if needle.is_empty() {
            return Vec::new();
        }

        let haystack = if case_sensitive {
            self.text.clone()
        } else {
            self.text.to_lowercase()
        };
        let needle = if case_sensitive {
            needle.to_string()
        } else {
            needle.to_lowercase()
        };

        if haystack.len() != self.text.len() {
            return self.find_slowly(&needle);
        }

        let mut found = Vec::new();
        let mut at = 0;
        while let Some(offset) = haystack[at..].find(&needle) {
            let start = at + offset;
            let end = start + needle.len();
            found.push(self.buffer.byte_to_char(start)..self.buffer.byte_to_char(end));
            at = start + needle.chars().next().map_or(1, |c| c.len_utf8());
        }
        found
    }

    fn find_slowly(&self, needle: &str) -> Vec<Range<usize>> {
        let chars: Vec<char> = self.text.chars().collect();
        let lowered: Vec<char> = chars
            .iter()
            .flat_map(|c| c.to_lowercase())
            .collect::<Vec<char>>();
        if lowered.len() != chars.len() {
            return Vec::new();
        }

        let wanted: Vec<char> = needle.chars().collect();
        let mut found = Vec::new();
        if wanted.is_empty() || wanted.len() > lowered.len() {
            return found;
        }

        for start in 0..=lowered.len() - wanted.len() {
            if lowered[start..start + wanted.len()] == wanted[..] {
                found.push(start..start + wanted.len());
            }
        }
        found
    }

    pub fn byte_range(&self, chars: Range<usize>) -> Range<usize> {
        self.buffer.char_to_byte(chars.start)..self.buffer.char_to_byte(chars.end)
    }

    pub fn char_range(&self, bytes: Range<usize>) -> Range<usize> {
        self.buffer.byte_to_char(bytes.start)..self.buffer.byte_to_char(bytes.end)
    }

    pub fn select_range(&mut self, range: Range<usize>) {
        let len = self.buffer.len_chars();
        self.selection = Selection::new(range.start.min(len), range.end.min(len));
        self.goal_column = None;
    }

    pub fn selected_text(&self) -> Option<String> {
        self.selected_bytes().map(|range| self.text[range].to_string())
    }

    pub fn line_text(&self) -> String {
        let point = self.buffer.offset_to_point(self.selection.head);
        let start = self.buffer.line_start(point.line);
        let end = start + self.buffer.line_len(point.line);
        let bytes = self.buffer.char_to_byte(start)..self.buffer.char_to_byte(end);
        let mut line = self.text[bytes].to_string();
        line.push('\n');
        line
    }

    pub fn undo(&mut self) -> bool {
        self.reshape(|buffer| buffer.undo().is_some())
    }

    pub fn redo(&mut self) -> bool {
        self.reshape(|buffer| buffer.redo().is_some())
    }

    pub fn commit(&mut self) {
        self.buffer.commit();
    }

    pub fn select_all(&mut self) {
        self.selection = Selection::new(0, self.buffer.len_chars());
        self.goal_column = None;
    }

    pub fn move_cursor(&mut self, motion: Motion, extend: bool) {
        let head = self.selection.head;
        let target = match motion {
            Motion::To(offset) => offset.min(self.buffer.len_chars()),
            Motion::Left => {
                if !extend && !self.selection.is_empty() {
                    self.selection.start()
                } else {
                    head.saturating_sub(1)
                }
            }
            Motion::Right => {
                if !extend && !self.selection.is_empty() {
                    self.selection.end()
                } else {
                    (head + 1).min(self.buffer.len_chars())
                }
            }
            Motion::Up => self.vertical(-1),
            Motion::Down => self.vertical(1),
            Motion::PageUp(rows) => self.vertical(-(rows as isize)),
            Motion::PageDown(rows) => self.vertical(rows as isize),
            Motion::WordLeft => self.word_boundary(head, -1),
            Motion::WordRight => self.word_boundary(head, 1),
            Motion::LineStart => {
                let point = self.buffer.offset_to_point(head);
                self.buffer.line_start(point.line)
            }
            Motion::LineEnd => {
                let point = self.buffer.offset_to_point(head);
                self.buffer.line_start(point.line) + self.buffer.line_len(point.line)
            }
            Motion::DocumentStart => 0,
            Motion::DocumentEnd => self.buffer.len_chars(),
        };

        let keep_goal = motion.keeps_goal_column();
        self.place(target, extend);
        if !keep_goal {
            self.goal_column = None;
        }
    }

    pub fn offset_at(&self, point: Point) -> usize {
        self.buffer.point_to_offset(point)
    }

    pub fn point_of(&self, offset: usize) -> Point {
        self.buffer.offset_to_point(offset)
    }

    fn place(&mut self, offset: usize, extend: bool) {
        let offset = offset.min(self.buffer.len_chars());
        self.selection = if extend {
            Selection::new(self.selection.anchor, offset)
        } else {
            Selection::cursor(offset)
        };
    }

    fn vertical(&mut self, rows: isize) -> usize {
        let point = self.buffer.offset_to_point(self.selection.head);
        let column = self.goal_column.unwrap_or(point.column);
        self.goal_column = Some(column);

        let last = self.buffer.len_lines().saturating_sub(1);
        let line = (point.line as isize + rows).clamp(0, last as isize) as usize;
        self.buffer.point_to_offset(Point::new(line, column))
    }

    fn word_boundary(&self, from: usize, direction: isize) -> usize {
        let chars: Vec<char> = self.buffer.text().chars().collect();
        let len = chars.len();
        let mut at = from;

        if direction < 0 {
            while at > 0 && class_of(chars[at - 1]) == CharClass::Whitespace {
                at -= 1;
            }
            if at == 0 {
                return 0;
            }
            let class = class_of(chars[at - 1]);
            while at > 0 && class_of(chars[at - 1]) == class {
                at -= 1;
            }
            at
        } else {
            while at < len && class_of(chars[at]) == CharClass::Whitespace {
                at += 1;
            }
            if at >= len {
                return len;
            }
            let class = class_of(chars[at]);
            while at < len && class_of(chars[at]) == class {
                at += 1;
            }
            at
        }
    }

    fn replace(&mut self, range: Range<usize>, text: &str) {
        let len = self.buffer.len_chars();
        let start = range.start.min(len);
        let end = range.end.min(len).max(start);
        if start == end && text.is_empty() {
            return;
        }

        let start_byte = self.buffer.char_to_byte(start);
        let end_byte = self.buffer.char_to_byte(end);
        let before = std::mem::take(&mut self.text);

        self.buffer.edit([Edit::replace(start..end, text)]);
        let after = self.buffer.text();

        if let Some(tree) = self.tree.as_mut() {
            tree.edit(
                &before,
                &after,
                start_byte,
                end_byte,
                start_byte + text.len(),
            );
            let _ = tree.parse(&after);
        }
        self.text = after;
        self.goal_column = None;
    }

    fn reshape(&mut self, change: impl FnOnce(&mut Buffer) -> bool) -> bool {
        if !change(&mut self.buffer) {
            return false;
        }
        self.text = self.buffer.text();
        if let Some(tree) = self.tree.as_mut() {
            tree.reset();
            let _ = tree.parse(&self.text);
        }
        let head = self.selection.head.min(self.buffer.len_chars());
        self.selection = Selection::cursor(head);
        self.goal_column = None;
        true
    }
}
