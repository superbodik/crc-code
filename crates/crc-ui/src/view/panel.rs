use crc_theme::Metrics;

use crate::geometry::Rect;
use crate::icon;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PanelTab {
    #[default]
    Problems,
    Output,
}

impl PanelTab {
    pub const ALL: [PanelTab; 2] = [PanelTab::Problems, PanelTab::Output];

    pub const fn title(self) -> &'static str {
        match self {
            PanelTab::Problems => "Проблемы",
            PanelTab::Output => "Вывод",
        }
    }

    pub const fn glyph(self) -> char {
        match self {
            PanelTab::Problems => icon::PROBLEMS,
            PanelTab::Output => icon::OUTPUT,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PanelView {
    pub tab: PanelTab,
    pub problems: Vec<Problem>,
    pub output: Vec<String>,
    pub scroll: usize,
    pub hovered: Option<usize>,
    pub selected: Option<usize>,
}

impl PanelView {
    pub fn rows(&self) -> usize {
        match self.tab {
            PanelTab::Problems => self.problems.len(),
            PanelTab::Output => self.output.len(),
        }
    }

    pub fn count(&self, tab: PanelTab) -> usize {
        match tab {
            PanelTab::Problems => self.problems.len(),
            PanelTab::Output => self.output.len(),
        }
    }

    pub fn empty_note(&self) -> &'static str {
        match self.tab {
            PanelTab::Problems => "Разбор проходит без ошибок",
            PanelTab::Output => "Пока нечего показать",
        }
    }
}

pub const TAB_PADDING: f32 = 10.0;
pub const TAB_GAP: f32 = 4.0;

#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    pub header: Rect,
    pub tabs: Vec<Rect>,
    pub body: Rect,
    pub rows: Vec<Rect>,
}

pub fn layout(panel: Rect, view: &PanelView, metrics: &Metrics, glyph_width: f32) -> Layout {
    let header = Rect::new(panel.x, panel.y, panel.width, metrics.row_height + 6.0);

    let mut tabs = Vec::with_capacity(PanelTab::ALL.len());
    let mut x = header.x + metrics.panel_padding;
    for tab in PanelTab::ALL {
        let letters = tab.title().chars().count() as f32;
        let count = view.count(tab);
        let badge = if count > 0 {
            count.to_string().chars().count() as f32 + 1.0
        } else {
            0.0
        };
        let width = (letters + badge) * glyph_width + TAB_PADDING * 2.0 + 16.0;
        tabs.push(Rect::new(x, header.y + 4.0, width, header.height - 8.0));
        x += width + TAB_GAP;
    }

    let body = Rect::new(
        panel.x,
        header.bottom(),
        panel.width,
        (panel.bottom() - header.bottom()).max(0.0),
    );

    let height = (metrics.row_height - 2.0).max(1.0);
    let visible = (body.height / height).floor() as usize;
    let mut rows = Vec::new();
    let mut y = body.y;
    for _ in 0..view.rows().saturating_sub(view.scroll).min(visible) {
        rows.push(Rect::new(body.x, y, body.width, height));
        y += height;
    }

    Layout {
        header,
        tabs,
        body,
        rows,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Tab(usize),
    Row(usize),
}

pub fn target_at(layout: &Layout, view: &PanelView, x: f32, y: f32) -> Option<Target> {
    if let Some(index) = layout.tabs.iter().position(|tab| tab.contains(x, y)) {
        return Some(Target::Tab(index));
    }

    let row = layout.rows.iter().position(|rect| rect.contains(x, y))?;
    let index = row + view.scroll;
    (index < view.rows()).then_some(Target::Row(index))
}

pub fn visible_rows(layout: &Layout) -> usize {
    layout.rows.len()
}
