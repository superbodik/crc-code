#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Density {
    Calm,
    #[default]
    Balanced,
    Dense,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    pub row_height: f32,
    pub panel_padding: f32,
    pub gap: f32,
    pub section_gap: f32,

    pub titlebar_height: f32,
    pub tabbar_height: f32,
    pub statusbar_height: f32,
    pub sidebar_width: f32,
    pub rail_width: f32,

    pub corner_radius: f32,
    pub corner_radius_small: f32,
    pub border_width: f32,

    pub gutter_width: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Affordances {
    pub minimap: bool,
    pub bottom_panel: bool,
    pub inline_diagnostics: bool,
    pub breadcrumbs: bool,
}

impl Metrics {
    pub fn scaled(self, factor: f32) -> Metrics {
        Metrics {
            row_height: self.row_height * factor,
            panel_padding: self.panel_padding * factor,
            gap: self.gap * factor,
            section_gap: self.section_gap * factor,
            titlebar_height: self.titlebar_height * factor,
            tabbar_height: self.tabbar_height * factor,
            statusbar_height: self.statusbar_height * factor,
            sidebar_width: self.sidebar_width * factor,
            rail_width: self.rail_width * factor,
            corner_radius: self.corner_radius * factor,
            corner_radius_small: self.corner_radius_small * factor,
            border_width: self.border_width * factor,
            gutter_width: self.gutter_width * factor,
        }
    }
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

    pub const fn label(self) -> &'static str {
        match self {
            Density::Calm => "Спокойно",
            Density::Balanced => "Сбалансированно",
            Density::Dense => "Максимум мощи",
        }
    }
}
