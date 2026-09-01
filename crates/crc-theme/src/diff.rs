use crate::color::Rgba;
use crate::palette;

/// Colours for the diff and review views.
///
/// Each side gets three tones — a wash behind the line, the text itself, and a
/// dimmed gutter number — so an added line reads as added at a glance without
/// the background fighting the code for attention.
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

    /// The gutter mark on a file with uncommitted changes.
    pub modified_marker: Rgba,
}

impl DiffTheme {
    pub const fn light() -> Self {
        Self {
            added_background: palette::GREEN_50,
            added_text: palette::GREEN_600,
            added_marker: palette::GREEN_500,
            added_line_number: palette::GREEN_200,

            removed_background: palette::RED_50,
            removed_text: palette::RED_600,
            removed_marker: palette::RED_500,
            removed_line_number: palette::RED_200,

            modified_marker: palette::AMBER_500,
        }
    }
}

impl Default for DiffTheme {
    fn default() -> Self {
        Self::light()
    }
}
