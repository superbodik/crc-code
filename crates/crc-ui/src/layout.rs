use crc_theme::Theme;

use crate::geometry::Rect;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShellState {
    pub sidebar_open: bool,
    pub aside_open: bool,
    pub aside_width: f32,
    pub panel_height: f32,
    pub minimap_width: f32,
}

impl Default for ShellState {
    fn default() -> Self {
        Self {
            sidebar_open: true,
            aside_open: false,
            aside_width: 320.0,
            panel_height: 200.0,
            minimap_width: 72.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shell {
    pub titlebar: Rect,
    pub rail: Option<Rect>,
    pub sidebar: Option<Rect>,
    pub tabs: Rect,
    pub breadcrumbs: Option<Rect>,
    pub gutter: Rect,
    pub buffer: Rect,
    pub minimap: Option<Rect>,
    pub panel: Option<Rect>,
    pub aside: Option<Rect>,
    pub statusbar: Option<Rect>,
}

const MIN_BUFFER_WIDTH: f32 = 240.0;
const MIN_BUFFER_HEIGHT: f32 = 120.0;

impl Shell {
    pub fn compute(window: Rect, theme: &Theme, state: &ShellState) -> Shell {
        let metrics = theme.metrics();
        let visible = theme.affordances();

        let (titlebar, body) = window.split_top(metrics.titlebar_height);

        let (statusbar, body) = if theme.zen {
            (None, body)
        } else {
            let (bar, rest) = body.split_bottom(metrics.statusbar_height);
            (Some(bar), rest)
        };

        let mut body = body;

        let rail = if !theme.zen && metrics.rail_width > 0.0 {
            let (rail, rest) = body.split_left(metrics.rail_width);
            body = rest;
            Some(rail)
        } else {
            None
        };

        let sidebar =
            if !theme.zen && state.sidebar_open && fits_width(&body, metrics.sidebar_width) {
                let (sidebar, rest) = body.split_left(metrics.sidebar_width);
                body = rest;
                Some(sidebar)
            } else {
                None
            };

        let aside = if !theme.zen && state.aside_open && fits_width(&body, state.aside_width) {
            let (aside, rest) = body.split_right(state.aside_width);
            body = rest;
            Some(aside)
        } else {
            None
        };

        let panel = if visible.bottom_panel && fits_height(&body, state.panel_height) {
            let (panel, rest) = body.split_bottom(state.panel_height);
            body = rest;
            Some(panel)
        } else {
            None
        };

        let (tabs, body_below_tabs) = body.split_top(metrics.tabbar_height);
        body = body_below_tabs;

        let breadcrumbs = if visible.breadcrumbs && fits_height(&body, metrics.row_height) {
            let (crumbs, rest) = body.split_top(metrics.row_height);
            body = rest;
            Some(crumbs)
        } else {
            None
        };

        let minimap = if visible.minimap && fits_width(&body, state.minimap_width) {
            let (minimap, rest) = body.split_right(state.minimap_width);
            body = rest;
            Some(minimap)
        } else {
            None
        };

        let (gutter, buffer) = body.split_left(metrics.gutter_width.min(body.width * 0.5));

        Shell {
            titlebar,
            rail,
            sidebar,
            tabs,
            breadcrumbs,
            gutter,
            buffer,
            minimap,
            panel,
            aside,
            statusbar,
        }
    }

    pub fn regions(&self) -> Vec<(&'static str, Rect)> {
        let mut regions = vec![("titlebar", self.titlebar), ("tabs", self.tabs)];
        for (name, rect) in [
            ("rail", self.rail),
            ("sidebar", self.sidebar),
            ("breadcrumbs", self.breadcrumbs),
            ("minimap", self.minimap),
            ("panel", self.panel),
            ("aside", self.aside),
            ("statusbar", self.statusbar),
        ] {
            if let Some(rect) = rect {
                regions.push((name, rect));
            }
        }
        regions.push(("gutter", self.gutter));
        regions.push(("buffer", self.buffer));
        regions
    }

    pub fn region_at(&self, x: f32, y: f32) -> Option<&'static str> {
        self.regions()
            .into_iter()
            .find(|(_, rect)| rect.contains(x, y))
            .map(|(name, _)| name)
    }
}

fn fits_width(body: &Rect, taken: f32) -> bool {
    body.width - taken >= MIN_BUFFER_WIDTH
}

fn fits_height(body: &Rect, taken: f32) -> bool {
    body.height - taken >= MIN_BUFFER_HEIGHT
}
