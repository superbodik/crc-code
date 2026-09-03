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
