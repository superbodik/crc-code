use crate::geometry::Rect;
use crate::icon;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Field,
    Previous,
    Next,
    MatchCase,
    Close,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FindView {
    pub query: String,
    pub total: usize,
    pub current: usize,
    pub match_case: bool,
    pub hovered: Option<Target>,
}

impl FindView {
    pub fn tally(&self) -> String {
        if self.query.is_empty() {
            return String::new();
        }
        if self.total == 0 {
            return "нет совпадений".to_string();
        }
        format!("{} из {}", self.current + 1, self.total)
    }

    pub fn step(&mut self, forward: bool) {
        if self.total == 0 {
            self.current = 0;
            return;
        }
        self.current = if forward {
            (self.current + 1) % self.total
        } else {
            (self.current + self.total - 1) % self.total
        };
    }
}

pub const WIDTH: f32 = 440.0;
pub const HEIGHT: f32 = 44.0;
pub const BUTTON: f32 = 26.0;
pub const PADDING: f32 = 10.0;
pub const TALLY: f32 = 96.0;
pub const MARGIN: f32 = 12.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    pub bar: Rect,
    pub field: Rect,
    pub tally: Rect,
    pub match_case: Rect,
    pub previous: Rect,
    pub next: Rect,
    pub close: Rect,
}

pub fn layout(buffer: Rect, scale: f32) -> Layout {
    let scale = scale.max(0.5);
    let width = (WIDTH * scale).min(buffer.width - 2.0 * MARGIN * scale);
    let height = HEIGHT * scale;

    let bar = Rect::new(
        buffer.right() - width - MARGIN * scale,
        buffer.y + MARGIN * scale,
        width.max(0.0),
        height,
    );

    let button = BUTTON * scale;
    let gap = 4.0 * scale;
    let inset = PADDING * scale;

    let close = Rect::new(
        bar.right() - inset - button,
        bar.y + (bar.height - button) / 2.0,
        button,
        button,
    );
    let next = Rect::new(close.x - gap - button, close.y, button, button);
    let previous = Rect::new(next.x - gap - button, close.y, button, button);
    let match_case = Rect::new(previous.x - gap - button, close.y, button, button);

    let tally_width = (TALLY * scale).min((match_case.x - bar.x - inset * 2.0).max(0.0));
    let tally = Rect::new(
        match_case.x - gap - tally_width,
        bar.y,
        tally_width,
        bar.height,
    );

    let field = Rect::new(
        bar.x + inset,
        bar.y + (bar.height - button - 4.0 * scale) / 2.0,
        (tally.x - bar.x - inset - gap).max(0.0),
        button + 4.0 * scale,
    );

    Layout {
        bar,
        field,
        tally,
        match_case,
        previous,
        next,
        close,
    }
}

pub fn glyph(target: Target) -> char {
    match target {
        Target::Previous => icon::CHEVRON_UP,
        Target::Next => icon::CHEVRON_DOWN,
        Target::MatchCase => icon::MATCH_CASE,
        Target::Close => icon::CLOSE,
        Target::Field => icon::SEARCH,
    }
}

pub fn target_at(layout: &Layout, x: f32, y: f32) -> Option<Target> {
    if !layout.bar.contains(x, y) {
        return None;
    }
    if layout.close.contains(x, y) {
        return Some(Target::Close);
    }
    if layout.next.contains(x, y) {
        return Some(Target::Next);
    }
    if layout.previous.contains(x, y) {
        return Some(Target::Previous);
    }
    if layout.match_case.contains(x, y) {
        return Some(Target::MatchCase);
    }
    if layout.field.contains(x, y) {
        return Some(Target::Field);
    }
    None
}
