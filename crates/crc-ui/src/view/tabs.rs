use crc_theme::TypeScale;

use crate::geometry::Rect;
use crate::view::state::Tab;

pub const PADDING: f32 = 12.0;
pub const CLOSE_SIZE: f32 = 14.0;
pub const CLOSE_GAP: f32 = 8.0;
const GLYPH_RATIO: f32 = 0.62;

pub fn width(tab: &Tab, scale: &TypeScale) -> f32 {
    let label = tab.name.chars().count() as f32 * scale.small * GLYPH_RATIO;
    PADDING * 2.0 + label + CLOSE_GAP + CLOSE_SIZE
}

pub fn widths(tabs: &[Tab], scale: &TypeScale) -> Vec<f32> {
    tabs.iter().map(|tab| width(tab, scale)).collect()
}

pub fn rects(bar: Rect, tabs: &[Tab], scale: &TypeScale) -> Vec<Rect> {
    let mut out = Vec::with_capacity(tabs.len());
    let mut x = bar.x;

    for tab in tabs {
        let width = width(tab, scale);
        if x + width > bar.right() {
            break;
        }
        out.push(Rect::new(x, bar.y, width, bar.height));
        x += width;
    }
    out
}

pub fn close_rect(tab: Rect) -> Rect {
    Rect::new(
        tab.right() - PADDING - CLOSE_SIZE,
        tab.y + (tab.height - CLOSE_SIZE) / 2.0,
        CLOSE_SIZE,
        CLOSE_SIZE,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabHit {
    Select(usize),
    Close(usize),
}

pub fn hit(bar: Rect, tabs: &[Tab], scale: &TypeScale, x: f32, y: f32) -> Option<TabHit> {
    if !bar.contains(x, y) {
        return None;
    }
    for (index, rect) in rects(bar, tabs, scale).into_iter().enumerate() {
        if !rect.contains(x, y) {
            continue;
        }
        return Some(if close_rect(rect).inset(-3.0).contains(x, y) {
            TabHit::Close(index)
        } else {
            TabHit::Select(index)
        });
    }
    None
}
