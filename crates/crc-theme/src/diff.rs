use crate::color::Rgba;
use crate::palette;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DiffTheme {
    pub added_background: Rgba,
    pub added_text: Rgba,
    pub added_marker: Rgba,
    pub added_line_number: Rgba,

    pub removed_background: Rgba,
    pub removed_text: Rgba,
    pub removed_marker: Rgba,
    pub removed_line_number: Rgba,

    pub modified_marker: Rgba,
}

impl DiffTheme {
    pub const fn light() -> Self {
        Self {
            added_background: palette::light::GREEN_50,
            added_text: palette::light::GREEN_600,
            added_marker: palette::light::GREEN_500,
            added_line_number: palette::light::GREEN_200,

            removed_background: palette::light::RED_50,
            removed_text: palette::light::RED_600,
            removed_marker: palette::light::RED_500,
            removed_line_number: palette::light::RED_200,

            modified_marker: palette::light::AMBER_500,
        }
    }

    pub const fn dark() -> Self {
        Self {
            added_background: palette::dark::GREEN_50,
            added_text: palette::dark::GREEN_600,
            added_marker: palette::dark::GREEN_500,
            added_line_number: palette::dark::GREEN_200,

            removed_background: palette::dark::RED_50,
            removed_text: palette::dark::RED_600,
            removed_marker: palette::dark::RED_500,
            removed_line_number: palette::dark::RED_200,

            modified_marker: palette::dark::AMBER_500,
        }
    }
}

impl Default for DiffTheme {
    fn default() -> Self {
        Self::light()
    }
}
