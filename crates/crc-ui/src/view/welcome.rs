use crate::geometry::Rect;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecentEntry {
    pub name: String,
    pub path: String,
    pub when: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WelcomeView {
    pub recent: Vec<RecentEntry>,
    pub hints: Vec<(String, String)>,
    pub hovered: Option<Target>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Recent(usize),
    OpenFolder,
}

pub const WIDTH: f32 = 560.0;
pub const MARK: f32 = 56.0;
pub const TITLE: f32 = 58.0;
pub const TAGLINE: f32 = 34.0;
pub const HEADING: f32 = 30.0;
pub const ROW: f32 = 46.0;
pub const BUTTON: f32 = 44.0;
pub const GAP: f32 = 24.0;
pub const HINT_ROW: f32 = 26.0;
pub const MAX_RECENT: usize = 5;

#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    pub mark: Rect,
    pub title: Rect,
    pub tagline: Rect,
    pub recent_heading: Rect,
    pub recent: Vec<Rect>,
    pub open_folder: Rect,
    pub hints: Vec<Rect>,
}

pub fn layout(window: Rect, view: &WelcomeView, scale: f32) -> Layout {
    let scale = scale.max(0.5);
    let width = (WIDTH * scale).min(window.width - 64.0 * scale);
    let shown = view.recent.len().min(MAX_RECENT);
    let hints = view.hints.len();

    let height = (MARK + TITLE + TAGLINE + GAP + HEADING) * scale
        + shown as f32 * ROW * scale
        + (GAP + BUTTON) * scale
        + if hints == 0 {
            0.0
        } else {
            (GAP + HEADING) * scale + hints as f32 * HINT_ROW * scale
        };

    let left = window.x + (window.width - width) / 2.0;
    let mut y = window.y + ((window.height - height) / 2.0).max(GAP * scale);

    let mark = Rect::new(left, y, MARK * scale, MARK * scale);
    y += (MARK + 12.0) * scale;

    let title = Rect::new(left, y, width, TITLE * scale);
    y += TITLE * scale;

    let tagline = Rect::new(left, y, width, TAGLINE * scale);
    y += (TAGLINE + GAP) * scale;

    let recent_heading = Rect::new(left, y, width, HEADING * scale);
    y += HEADING * scale;

    let mut recent = Vec::with_capacity(shown);
    for _ in 0..shown {
        recent.push(Rect::new(left, y, width, ROW * scale));
        y += ROW * scale;
    }

    y += GAP * scale;
    let open_folder = Rect::new(left, y, width, BUTTON * scale);
    y += (BUTTON + GAP) * scale;

    let mut hint_rects = Vec::with_capacity(hints);
    if hints > 0 {
        y += HEADING * scale;
        for _ in 0..hints {
            hint_rects.push(Rect::new(left, y, width, HINT_ROW * scale));
            y += HINT_ROW * scale;
        }
    }

    Layout {
        mark,
        title,
        tagline,
        recent_heading,
        recent: recent.clone(),
        open_folder,
        hints: hint_rects,
    }
}

pub fn target_at(layout: &Layout, x: f32, y: f32) -> Option<Target> {
    if layout.open_folder.contains(x, y) {
        return Some(Target::OpenFolder);
    }
    layout
        .recent
        .iter()
        .position(|rect| rect.contains(x, y))
        .map(Target::Recent)
}
