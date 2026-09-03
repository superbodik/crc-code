use crc_theme::Metrics;

use crate::geometry::Rect;
use crate::icon;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RailAction {
    Explorer,
    Search,
    Settings,
}

impl RailAction {
    pub const ALL: [RailAction; 3] = [
        RailAction::Explorer,
        RailAction::Search,
        RailAction::Settings,
    ];

    pub const fn glyph(self) -> char {
        match self {
            RailAction::Explorer => icon::EXPLORER,
            RailAction::Search => icon::SEARCH,
            RailAction::Settings => icon::GEAR,
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            RailAction::Explorer => "Проводник",
            RailAction::Search => "Поиск",
            RailAction::Settings => "Настройки",
        }
    }
}

pub const BUTTON: f32 = 34.0;
pub const GAP: f32 = 6.0;

pub fn button(rail: Rect, metrics: &Metrics, index: usize) -> Rect {
    let size = BUTTON.min(rail.width - 6.0).max(0.0);
    Rect::new(
        rail.x + (rail.width - size) / 2.0,
        rail.y + metrics.panel_padding + index as f32 * (size + GAP),
        size,
        size,
    )
}

pub fn action_at(rail: Rect, metrics: &Metrics, x: f32, y: f32) -> Option<RailAction> {
    if !rail.contains(x, y) {
        return None;
    }
    RailAction::ALL
        .into_iter()
        .enumerate()
        .find(|(index, _)| button(rail, metrics, *index).contains(x, y))
        .map(|(_, action)| action)
}
