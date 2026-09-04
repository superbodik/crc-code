use crc_text::Point;
use crc_theme::Metrics;

use crate::geometry::Rect;
use crate::view::state::CodeMetrics;

pub fn buffer_point(
    buffer: Rect,
    metrics: CodeMetrics,
    scroll_line: usize,
    x: f32,
    y: f32,
) -> Point {
    let row = if metrics.line_height > 0.0 {
        ((y - buffer.y) / metrics.line_height).floor().max(0.0) as usize
    } else {
        0
    };
    let column = if metrics.char_width > 0.0 {
        ((x - buffer.x) / metrics.char_width).round().max(0.0) as usize
    } else {
        0
    };
    Point::new(scroll_line + row, column)
}

pub fn explorer_header_height(metrics: &Metrics) -> f32 {
    metrics.row_height + 8.0
}

pub fn explorer_row(sidebar: Rect, metrics: &Metrics, y: f32) -> Option<usize> {
    let top = sidebar.y + explorer_header_height(metrics);
    if y < top || y >= sidebar.bottom() || metrics.row_height <= 0.0 {
        return None;
    }
    Some(((y - top) / metrics.row_height).floor() as usize)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplorerButton {
    NewFolder,
    NewFile,
}

impl ExplorerButton {
    pub const ALL: [ExplorerButton; 2] = [ExplorerButton::NewFolder, ExplorerButton::NewFile];

    pub const fn glyph(self) -> char {
        match self {
            ExplorerButton::NewFolder => crate::icon::NEW_FOLDER,
            ExplorerButton::NewFile => crate::icon::NEW_FILE,
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            ExplorerButton::NewFolder => "Новая папка",
            ExplorerButton::NewFile => "Новый файл",
        }
    }
}

pub const EXPLORER_BUTTON: f32 = 24.0;

pub fn explorer_button(sidebar: Rect, metrics: &Metrics, index: usize) -> Rect {
    let header = explorer_header_height(metrics);
    let size = EXPLORER_BUTTON;
    Rect::new(
        sidebar.right() - metrics.panel_padding - (index as f32 + 1.0) * (size + 2.0),
        sidebar.y + (header - size) / 2.0,
        size,
        size,
    )
}

pub fn explorer_button_at(
    sidebar: Rect,
    metrics: &Metrics,
    x: f32,
    y: f32,
) -> Option<ExplorerButton> {
    if y >= sidebar.y + explorer_header_height(metrics) {
        return None;
    }
    ExplorerButton::ALL
        .into_iter()
        .enumerate()
        .find(|(index, _)| explorer_button(sidebar, metrics, *index).contains(x, y))
        .map(|(_, button)| button)
}
