/// How much the interface shows at once.
///
/// Onboarding asks the question directly — "насколько шумно должно быть?" — and
/// the answer is one value, not a dozen toggles. Every panel reads its metrics
/// from here, so switching profiles moves the whole shell together instead of
/// leaving one pane at the old rhythm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Density {
    /// One sidebar, hints on request, no popups. The learning profile.
    Calm,
    #[default]
    Balanced,
    /// Three panels, minimap, inline diagnostics, terminal below.
    Dense,
}

/// Sizes and spacing, in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// A row in a list: file tree, search results, palette.
    pub row_height: f32,
    /// Inside a panel, from its edge to its content.
    pub panel_padding: f32,
    /// Between elements in a row.
    pub gap: f32,
    /// Between sections in a panel.
    pub section_gap: f32,

    pub titlebar_height: f32,
    pub tabbar_height: f32,
    pub statusbar_height: f32,
    pub sidebar_width: f32,
    /// The activity rail down the far edge.
    pub rail_width: f32,

    pub corner_radius: f32,
    /// Radius for small controls: buttons, badges, chips.
    pub corner_radius_small: f32,
    pub border_width: f32,

    /// Width of the gutter holding line numbers and diff marks.
    pub gutter_width: f32,
}

/// What the profile turns on, beyond the sizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Affordances {
    pub minimap: bool,
    pub bottom_panel: bool,
    pub inline_diagnostics: bool,
    /// Breadcrumbs above the buffer.
    pub breadcrumbs: bool,
}

impl Density {
    pub const fn metrics(self) -> Metrics {
        match self {
            Density::Calm => Metrics {
                row_height: 28.0,
                panel_padding: 16.0,
                gap: 10.0,
                section_gap: 20.0,
                titlebar_height: 40.0,
                tabbar_height: 36.0,
                statusbar_height: 24.0,
                sidebar_width: 260.0,
                rail_width: 0.0,
                corner_radius: 12.0,
                corner_radius_small: 8.0,
                border_width: 1.0,
                gutter_width: 52.0,
            },
            Density::Balanced => Metrics {
                row_height: 24.0,
                panel_padding: 12.0,
                gap: 8.0,
                section_gap: 16.0,
                titlebar_height: 40.0,
                tabbar_height: 34.0,
                statusbar_height: 22.0,
                sidebar_width: 240.0,
                rail_width: 44.0,
                corner_radius: 10.0,
                corner_radius_small: 7.0,
                border_width: 1.0,
                gutter_width: 48.0,
            },
            Density::Dense => Metrics {
                row_height: 20.0,
                panel_padding: 8.0,
                gap: 6.0,
                section_gap: 12.0,
                titlebar_height: 36.0,
                tabbar_height: 30.0,
                statusbar_height: 20.0,
                sidebar_width: 220.0,
                rail_width: 40.0,
                corner_radius: 8.0,
                corner_radius_small: 6.0,
                border_width: 1.0,
                gutter_width: 44.0,
            },
        }
    }

    pub const fn affordances(self) -> Affordances {
        match self {
            Density::Calm => Affordances {
                minimap: false,
                bottom_panel: false,
                inline_diagnostics: false,
                breadcrumbs: false,
            },
            Density::Balanced => Affordances {
                minimap: true,
                bottom_panel: true,
                inline_diagnostics: false,
                breadcrumbs: true,
            },
            Density::Dense => Affordances {
                minimap: true,
                bottom_panel: true,
                inline_diagnostics: true,
                breadcrumbs: true,
            },
        }
    }

    /// The label shown in settings and onboarding.
    pub const fn label(self) -> &'static str {
        match self {
            Density::Calm => "Спокойно",
            Density::Balanced => "Сбалансированно",
            Density::Dense => "Максимум мощи",
        }
    }
}
