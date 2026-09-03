use crc_theme::{Brand, Weight};

use crate::geometry::Rect;
use crate::gpu::{Frame, Quad, TextAlign, TextRun};

pub const RADIUS: f32 = 0.20;
pub const CUT: f32 = 0.26;
pub const CLEAR_SPACE: f32 = 0.12;
pub const CARET_WIDTH: f32 = 0.065;
pub const CARET_HEIGHT: f32 = 0.44;
pub const CARET_LEFT: f32 = 0.647;
pub const CARET_TOP: f32 = 0.28;
pub const GLYPH_CENTER: f32 = 0.4125;
pub const GLYPH_SIZE: f32 = 0.57;

pub const CUT_BELOW: f32 = 24.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mark {
    pub block: Rect,
    pub caret: Rect,
    pub glyph_center: f32,
    pub glyph_size: f32,
    pub radius: f32,
    pub cut: f32,
}

pub fn mark(side: f32, x: f32, y: f32) -> Mark {
    let side = side.max(0.0);
    Mark {
        block: Rect::new(x, y, side, side),
        caret: Rect::new(
            x + side * CARET_LEFT,
            y + side * CARET_TOP,
            side * CARET_WIDTH,
            side * CARET_HEIGHT,
        ),
        glyph_center: x + side * GLYPH_CENTER,
        glyph_size: side * GLYPH_SIZE,
        radius: side * RADIUS,
        cut: if side >= CUT_BELOW { side * CUT } else { 0.0 },
    }
}

pub fn clear_space(side: f32) -> f32 {
    side * CLEAR_SPACE
}

pub fn draw(frame: &mut Frame, mark: Mark, brand: Brand) {
    if mark.block.is_empty() {
        return;
    }

    frame.quad(Quad::filled(mark.block, brand.mark).rounded(mark.radius));

    let glyph = Rect::new(
        mark.glyph_center - mark.block.width,
        mark.block.y,
        mark.block.width * 2.0,
        mark.block.height,
    );
    frame.text(
        TextRun::new("C", glyph, mark.glyph_size, brand.glyph)
            .weight(Weight::Semibold)
            .align(TextAlign::Center)
            .line_height(mark.block.height),
    );

    frame.quad(Quad::filled(mark.caret, brand.caret));
}
