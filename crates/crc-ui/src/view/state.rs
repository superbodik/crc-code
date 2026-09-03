use std::ops::Range;

use crc_theme::Highlight;

use crate::view::controls::WindowControl;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tab {
    pub name: String,
    pub active: bool,
    pub modified: bool,
}

impl Tab {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            active: false,
            modified: false,
        }
    }

    pub fn active(mut self) -> Self {
        self.active = true;
        self
    }

    pub fn modified(mut self) -> Self {
        self.modified = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    pub name: String,
    pub depth: usize,
    pub is_dir: bool,
    pub selected: bool,
    pub modified: bool,
}

impl FileEntry {
    pub fn file(name: impl Into<String>, depth: usize) -> Self {
        Self {
            name: name.into(),
            depth,
            is_dir: false,
            selected: false,
            modified: false,
        }
    }

    pub fn dir(name: impl Into<String>, depth: usize) -> Self {
        Self {
            name: name.into(),
            depth,
            is_dir: true,
            selected: false,
            modified: false,
        }
    }

    pub fn selected(mut self) -> Self {
        self.selected = true;
        self
    }

    pub fn modified(mut self) -> Self {
        self.modified = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct EditorView {
    pub project: String,
    pub branch: String,
    pub tabs: Vec<Tab>,
    pub files: Vec<FileEntry>,
    pub text: String,
    pub highlights: Vec<(Range<usize>, Highlight)>,
    pub cursor_line: usize,
    pub cursor_column: usize,
    pub scroll_line: usize,
    pub language: String,
    pub problems: usize,
    pub focused: bool,
    pub maximized: bool,
    pub hovered_control: Option<WindowControl>,
    pub hovered_tab: Option<usize>,
    pub selection: Option<Range<usize>>,
    pub dirty: bool,
}

impl EditorView {
    pub fn line_count(&self) -> usize {
        self.text.lines().count().max(1)
    }

    pub fn visible(&self, rows: usize) -> VisibleText {
        let mut start = None;
        let mut end = None;
        let mut offset = 0usize;

        for (line, segment) in self.text.split_inclusive('\n').enumerate() {
            if line == self.scroll_line {
                start = Some(offset);
            }
            if line == self.scroll_line + rows {
                end = Some(offset);
                break;
            }
            offset += segment.len();
        }

        let start = start.unwrap_or(self.text.len());
        let end = end.unwrap_or(self.text.len()).max(start);

        let spans = self
            .highlights
            .iter()
            .filter(|(range, _)| range.end > start && range.start < end)
            .map(|(range, highlight)| {
                (
                    range.start.max(start) - start..range.end.min(end) - start,
                    *highlight,
                )
            })
            .collect();

        VisibleText {
            text: self.text[start..end].to_string(),
            spans,
            first_line: self.scroll_line,
            byte_start: start,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VisibleText {
    pub text: String,
    pub spans: Vec<(Range<usize>, Highlight)>,
    pub first_line: usize,
    pub byte_start: usize,
}

impl VisibleText {
    pub fn local(&self, range: &Range<usize>) -> Option<Range<usize>> {
        let end = self.byte_start + self.text.len();
        if range.end <= self.byte_start || range.start >= end {
            return None;
        }
        Some(
            range.start.max(self.byte_start) - self.byte_start
                ..range.end.min(end) - self.byte_start,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CodeMetrics {
    pub char_width: f32,
    pub line_height: f32,
}

impl CodeMetrics {
    pub fn rows(&self, height: f32) -> usize {
        if self.line_height <= 0.0 {
            return 0;
        }
        (height / self.line_height).floor().max(0.0) as usize
    }
}

impl Default for CodeMetrics {
    fn default() -> Self {
        Self {
            char_width: 7.8,
            line_height: 20.0,
        }
    }
}
