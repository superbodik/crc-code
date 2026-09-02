use crate::color::Rgba;
use crate::palette;

pub const CONTROL_RING: f32 = 0.25;

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

    pub control_close: Rgba,
    pub control_minimize: Rgba,
    pub control_maximize: Rgba,
    pub control_idle: Rgba,

    pub success: Rgba,
    pub warning: Rgba,
    pub danger: Rgba,
    pub info: Rgba,
}

impl Chrome {
    pub const fn light() -> Self {
        Self {
            backdrop: palette::light::NEUTRAL_250,
            surface: palette::light::NEUTRAL_75,
            panel: palette::light::NEUTRAL_150,
            raised: palette::light::NEUTRAL_0,
            hover: palette::light::NEUTRAL_200,
            selected: palette::light::NEUTRAL_300,

            border: palette::light::NEUTRAL_300,
            divider: palette::light::NEUTRAL_350,

            text: palette::light::NEUTRAL_850,
            text_strong: palette::light::NEUTRAL_900,
            text_muted: palette::light::NEUTRAL_600,
            text_faint: palette::light::NEUTRAL_550,
            text_on_accent: palette::light::NEUTRAL_0,

            accent: palette::light::ACCENT_500,
            accent_solid: palette::light::ACCENT_600,
            accent_hover: palette::light::ACCENT_700,
            accent_wash: palette::light::ACCENT_50,
            focus: palette::light::ACCENT_500,

            control_close: palette::light::TRAFFIC_RED,
            control_minimize: palette::light::TRAFFIC_AMBER,
            control_maximize: palette::light::TRAFFIC_GREEN,
            control_idle: palette::light::NEUTRAL_450,

            success: palette::light::GREEN_500,
            warning: palette::light::AMBER_500,
            danger: palette::light::RED_500,
            info: palette::light::BLUE_500,
        }
    }

    pub const fn dark() -> Self {
        Self {
            backdrop: palette::dark::NEUTRAL_0,
            surface: palette::dark::NEUTRAL_75,
            panel: palette::dark::NEUTRAL_100,
            raised: palette::dark::NEUTRAL_200,
            hover: palette::dark::NEUTRAL_250,
            selected: palette::dark::NEUTRAL_350,

            border: palette::dark::NEUTRAL_300,
            divider: palette::dark::NEUTRAL_200,

            text: palette::dark::NEUTRAL_850,
            text_strong: palette::dark::NEUTRAL_900,
            text_muted: palette::dark::NEUTRAL_650,
            text_faint: palette::dark::NEUTRAL_550,
            text_on_accent: palette::dark::NEUTRAL_0,

            accent: palette::dark::ACCENT_500,
            accent_solid: palette::dark::ACCENT_500,
            accent_hover: palette::dark::ACCENT_600,
            accent_wash: palette::dark::ACCENT_50,
            focus: palette::dark::ACCENT_500,

            control_close: palette::dark::TRAFFIC_RED,
            control_minimize: palette::dark::TRAFFIC_AMBER,
            control_maximize: palette::dark::TRAFFIC_GREEN,
            control_idle: palette::dark::NEUTRAL_450,

            success: palette::dark::GREEN_500,
            warning: palette::dark::AMBER_500,
            danger: palette::dark::RED_500,
            info: palette::dark::BLUE_500,
        }
    }
}

impl Default for Chrome {
    fn default() -> Self {
        Self::light()
    }
}
