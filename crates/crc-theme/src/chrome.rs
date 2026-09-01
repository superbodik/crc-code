use crate::color::Rgba;
use crate::palette;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Chrome {
    pub backdrop: Rgba,
    pub surface: Rgba,
    pub panel: Rgba,
    pub raised: Rgba,
    pub hover: Rgba,
    pub selected: Rgba,

    pub border: Rgba,
    pub divider: Rgba,

    pub text: Rgba,
    pub text_strong: Rgba,
    pub text_muted: Rgba,
    pub text_faint: Rgba,
    pub text_on_accent: Rgba,

    pub accent: Rgba,
    pub accent_solid: Rgba,
    pub accent_hover: Rgba,
    pub accent_wash: Rgba,
    pub focus: Rgba,

    pub success: Rgba,
    pub warning: Rgba,
    pub danger: Rgba,
    pub info: Rgba,
}

impl Chrome {
    pub const fn light() -> Self {
        Self {
            backdrop: palette::NEUTRAL_250,
            surface: palette::NEUTRAL_75,
            panel: palette::NEUTRAL_150,
            raised: palette::NEUTRAL_0,
            hover: palette::NEUTRAL_200,
            selected: palette::NEUTRAL_300,

            border: palette::NEUTRAL_300,
            divider: palette::NEUTRAL_350,

            text: palette::NEUTRAL_850,
            text_strong: palette::NEUTRAL_900,
            text_muted: palette::NEUTRAL_600,
            text_faint: palette::NEUTRAL_550,
            text_on_accent: palette::NEUTRAL_0,

            accent: palette::ACCENT_500,
            accent_solid: palette::ACCENT_600,
            accent_hover: palette::ACCENT_700,
            accent_wash: palette::ACCENT_50,
            focus: palette::ACCENT_500,

            success: palette::GREEN_500,
            warning: palette::AMBER_500,
            danger: palette::RED_500,
            info: palette::BLUE_500,
        }
    }
}

impl Default for Chrome {
    fn default() -> Self {
        Self::light()
    }
}
