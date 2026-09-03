use std::path::PathBuf;

use crc_theme::Metrics;

use crate::geometry::Rect;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchRow {
    File { path: PathBuf, hits: usize },
    Line { path: PathBuf, line: u64, text: String },
}

impl SearchRow {
    pub fn path(&self) -> &PathBuf {
        match self {
            SearchRow::File { path, .. } | SearchRow::Line { path, .. } => path,
        }
    }

    pub fn is_file(&self) -> bool {
        matches!(self, SearchRow::File { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SearchView {
    pub query: String,
    pub rows: Vec<SearchRow>,
    pub files: usize,
    pub hits: usize,
    pub match_case: bool,
    pub selected: Option<usize>,
    pub hovered: Option<usize>,
    pub scroll: usize,
    pub searched: bool,
}

impl SearchView {
    pub fn tally(&self) -> String {
        if self.query.trim().is_empty() {
            return "Введи, что искать".to_string();
        }
        if !self.searched {
            return "Ищу...".to_string();
        }
        if self.hits == 0 {
            return "Ничего не нашлось".to_string();
        }
        format!("{} в {} файлах", self.hits, self.files)
    }

    pub fn fold(results: &[(PathBuf, Vec<(u64, String)>)]) -> (Vec<SearchRow>, usize, usize) {
        let mut rows = Vec::new();
        let mut hits = 0;

        for (path, lines) in results {
            rows.push(SearchRow::File {
                path: path.clone(),
                hits: lines.len(),
            });
            hits += lines.len();

            for (line, text) in lines {
                rows.push(SearchRow::Line {
                    path: path.clone(),
                    line: *line,
                    text: text.trim_end().to_string(),
                });
            }
        }

        (rows, results.len(), hits)
    }
}

pub const HEADER: f32 = 34.0;
pub const FIELD: f32 = 30.0;
pub const TALLY: f32 = 22.0;
pub const PADDING: f32 = 10.0;

#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    pub header: Rect,
    pub field: Rect,
    pub match_case: Rect,
    pub tally: Rect,
    pub list: Rect,
    pub rows: Vec<Rect>,
}

pub fn layout(sidebar: Rect, view: &SearchView, metrics: &Metrics) -> Layout {
    let header = Rect::new(sidebar.x, sidebar.y, sidebar.width, HEADER);

    let button = FIELD - 6.0;
    let field = Rect::new(
        sidebar.x + PADDING,
        header.bottom(),
        (sidebar.width - PADDING * 2.0 - button - 4.0).max(0.0),
        FIELD,
    );
    let match_case = Rect::new(
        field.right() + 4.0,
        field.y + (FIELD - button) / 2.0,
        button,
        button,
    );
    let tally = Rect::new(sidebar.x + PADDING, field.bottom(), field.width, TALLY);

    let list = Rect::new(
        sidebar.x,
        tally.bottom() + 4.0,
        sidebar.width,
        (sidebar.bottom() - tally.bottom() - 4.0).max(0.0),
    );

    let height = metrics.row_height.max(1.0);
    let visible = (list.height / height).floor() as usize;
    let mut rows = Vec::new();
    let mut y = list.y;
    for _ in 0..view.rows.len().saturating_sub(view.scroll).min(visible) {
        rows.push(Rect::new(list.x, y, list.width, height));
        y += height;
    }

    Layout {
        header,
        field,
        match_case,
        tally,
        list,
        rows,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Field,
    MatchCase,
    Row(usize),
}

pub fn target_at(layout: &Layout, view: &SearchView, x: f32, y: f32) -> Option<Target> {
    if layout.match_case.contains(x, y) {
        return Some(Target::MatchCase);
    }
    if layout.field.contains(x, y) {
        return Some(Target::Field);
    }

    let row = layout.rows.iter().position(|rect| rect.contains(x, y))?;
    let index = row + view.scroll;
    (index < view.rows.len()).then_some(Target::Row(index))
}

pub fn visible_rows(layout: &Layout) -> usize {
    layout.rows.len()
}
