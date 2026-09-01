use crate::color::Rgba;
use crate::palette;

/// Colours for everything that is not code: panels, borders, labels, controls.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Chrome {
    /// Behind the window — the desktop canvas.
    pub backdrop: Rgba,
    /// The window body and the editor surface.
    pub surface: Rgba,
    /// Sidebars, the title bar, the status bar.
    pub panel: Rgba,
    /// Raised things on top of a panel: inputs, cards, the palette.
    pub raised: Rgba,
    /// A row under the pointer.
    pub hover: Rgba,
    /// The selected row or tab.
    pub selected: Rgba,

    pub border: Rgba,
    /// Divider between panes, quieter than a border.
    pub divider: Rgba,

    /// Body text.
    pub text: Rgba,
    /// Headings and anything that must not be missed.
    pub text_strong: Rgba,
    /// Labels, paths, timestamps.
    pub text_muted: Rgba,
    /// Disabled controls and placeholder text.
    pub text_faint: Rgba,
    /// Text on top of `accent`.
    pub text_on_accent: Rgba,

    /// The caret, the active tab underline, focus rings — marks, not text.
    pub accent: Rgba,
    /// Filled buttons. Darker than `accent`, because white sits on it: the
    /// lighter violet only reaches 4.35:1 against white, which is under the bar
    /// for a button label.
    pub accent_solid: Rgba,
    pub accent_hover: Rgba,
    /// A wash behind selected accent rows and match highlights.
    pub accent_wash: Rgba,
    /// Focus rings.
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
