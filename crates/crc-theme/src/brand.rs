use crate::color::Rgba;

pub const MARK: Rgba = Rgba::hex(0x4aa8ff);
pub const INK: Rgba = Rgba::hex(0x14171c);
pub const PAPER: Rgba = Rgba::hex(0xf5f6f8);
pub const TEAM: Rgba = Rgba::hex(0x00c48f);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Brand {
    pub mark: Rgba,
    pub glyph: Rgba,
    pub caret: Rgba,
}

impl Brand {
    pub const fn colour() -> Self {
        Self {
            mark: MARK,
            glyph: INK,
            caret: INK,
        }
    }

    pub const fn on_dark() -> Self {
        Self {
            mark: INK,
            glyph: PAPER,
            caret: MARK,
        }
    }

    pub const fn monochrome() -> Self {
        Self {
            mark: Rgba::hex(0xeeeff2),
            glyph: Rgba::hex(0x1f232a),
            caret: Rgba::hex(0x1f232a),
        }
    }
}

impl Default for Brand {
    fn default() -> Self {
        Self::colour()
    }
}
