use crate::geometry::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Section {
    #[default]
    Appearance,
    Keys,
}

impl Section {
    pub const ALL: [Section; 2] = [Section::Appearance, Section::Keys];

    pub const fn title(self) -> &'static str {
        match self {
            Section::Appearance => "Внешний вид",
            Section::Keys => "Клавиши",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toggle {
    pub id: String,
    pub label: String,
    pub note: String,
    pub on: bool,
}

impl Toggle {
    pub fn new(id: &str, label: &str, note: &str, on: bool) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            note: note.to_string(),
            on,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingRow {
    pub command: String,
    pub title: String,
    pub keys: String,
    pub clash: Option<String>,
    pub changed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Section(usize),
    Toggle(usize),
    Binding(usize),
    Search,
    Reset,
    Close,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SettingsView {
    pub section: Section,
    pub query: String,
    pub toggles: Vec<Toggle>,
    pub bindings: Vec<BindingRow>,
    pub capturing: Option<usize>,
    pub hovered: Option<Target>,
    pub scroll: usize,
}

impl SettingsView {
    pub fn shown(&self) -> Vec<usize> {
        match self.section {
            Section::Appearance => (0..self.toggles.len()).collect(),
            Section::Keys => self
                .bindings
                .iter()
                .enumerate()
                .filter(|(_, row)| matches(&self.query, row))
                .map(|(index, _)| index)
                .collect(),
        }
    }

    pub fn rows(&self) -> usize {
        self.shown().len()
    }

    pub fn touched(&self) -> bool {
        self.bindings.iter().any(|row| row.changed)
    }
}

pub fn matches(query: &str, row: &BindingRow) -> bool {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        return true;
    }
    row.title.to_lowercase().contains(&needle) || row.keys.to_lowercase().contains(&needle)
}

pub fn mark_clashes(rows: &mut [BindingRow]) {
    let taken: Vec<(String, String)> = rows
        .iter()
        .filter(|row| !row.keys.is_empty())
        .map(|row| (row.keys.to_lowercase(), row.title.clone()))
        .collect();

    for row in rows.iter_mut() {
        row.clash = None;
        if row.keys.is_empty() {
            continue;
        }
        let mine = row.keys.to_lowercase();
        let others: Vec<&String> = taken
            .iter()
            .filter(|(keys, title)| keys == &mine && title != &row.title)
            .map(|(_, title)| title)
            .collect();
        if let Some(other) = others.first() {
            row.clash = Some((*other).clone());
        }
    }
}

pub const WIDTH: f32 = 780.0;
pub const HEIGHT: f32 = 560.0;
pub const SIDEBAR: f32 = 190.0;
pub const HEADER: f32 = 56.0;
pub const SECTION_ROW: f32 = 38.0;
pub const ROW: f32 = 54.0;
pub const PADDING: f32 = 20.0;
pub const KEYCAP: f32 = 132.0;
pub const SCROLLBAR: f32 = 4.0;
pub const GUTTER: f32 = 10.0;
pub const SEARCH: f32 = 36.0;
pub const RESET: f32 = 108.0;

#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    pub panel: Rect,
    pub header: Rect,
    pub close: Rect,
    pub sidebar: Rect,
    pub sections: Vec<Rect>,
    pub body: Rect,
    pub search: Option<Rect>,
    pub reset: Option<Rect>,
    pub rows: Vec<Rect>,
    pub thumb: Option<Rect>,
}

pub fn layout(window: Rect, view: &SettingsView, scale: f32) -> Layout {
    let scale = scale.max(0.5);
    let width = (WIDTH * scale).min(window.width - 48.0 * scale);
    let height = (HEIGHT * scale).min(window.height - 48.0 * scale);

    let panel = Rect::new(
        window.x + (window.width - width) / 2.0,
        window.y + (window.height - height) / 2.0,
        width,
        height,
    );

    let (header, rest) = panel.split_top(HEADER * scale);
    let close = Rect::new(
        header.right() - (PADDING + 20.0) * scale,
        header.y + (header.height - 20.0 * scale) / 2.0,
        20.0 * scale,
        20.0 * scale,
    );

    let (sidebar, body) = rest.split_left(SIDEBAR * scale);

    let mut sections = Vec::with_capacity(Section::ALL.len());
    let mut y = sidebar.y + PADDING * scale;
    for _ in Section::ALL {
        sections.push(Rect::new(
            sidebar.x + 10.0 * scale,
            y,
            sidebar.width - 20.0 * scale,
            SECTION_ROW * scale,
        ));
        y += SECTION_ROW * scale;
    }

    let inner = body.inset_by(PADDING * scale, PADDING * scale);

    let (search, reset, list) = match view.section {
        Section::Appearance => (None, None, inner),
        Section::Keys => {
            let (top, list) = inner.split_top((SEARCH + GUTTER) * scale);
            let field = Rect::new(
                top.x,
                top.y,
                top.width - (RESET + GUTTER) * scale,
                SEARCH * scale,
            );
            let button = Rect::new(
                top.right() - RESET * scale,
                top.y,
                RESET * scale,
                SEARCH * scale,
            );
            (Some(field), Some(button), list)
        }
    };

    let visible = ((list.height / (ROW * scale)).floor() as usize).max(1);
    let total = view.rows();
    let hidden = total.saturating_sub(visible);
    let row_width = if hidden > 0 {
        list.width - (GUTTER + SCROLLBAR) * scale
    } else {
        list.width
    };

    let mut rows = Vec::new();
    let mut row_y = list.y;
    for _ in 0..total.saturating_sub(view.scroll).min(visible) {
        rows.push(Rect::new(list.x, row_y, row_width, ROW * scale));
        row_y += ROW * scale;
    }

    let thumb = (hidden > 0).then(|| {
        let track = visible as f32 * ROW * scale;
        let height = (track * visible as f32 / total as f32).max(32.0 * scale);
        let travel = view.scroll.min(hidden) as f32 / hidden as f32;
        Rect::new(
            list.right() - SCROLLBAR * scale,
            list.y + (track - height) * travel,
            SCROLLBAR * scale,
            height,
        )
    });

    Layout {
        panel,
        header,
        close,
        sidebar,
        sections,
        body,
        search,
        reset,
        rows,
        thumb,
    }
}

pub fn visible_rows(layout: &Layout) -> usize {
    layout.rows.len()
}

pub fn target_at(layout: &Layout, view: &SettingsView, x: f32, y: f32) -> Option<Target> {
    if !layout.panel.contains(x, y) {
        return None;
    }
    if layout.close.inset(-6.0).contains(x, y) {
        return Some(Target::Close);
    }
    if let Some(index) = layout.sections.iter().position(|rect| rect.contains(x, y)) {
        return Some(Target::Section(index));
    }
    if layout.reset.is_some_and(|rect| rect.contains(x, y)) {
        return Some(Target::Reset);
    }
    if layout.search.is_some_and(|rect| rect.contains(x, y)) {
        return Some(Target::Search);
    }

    let row = layout.rows.iter().position(|rect| rect.contains(x, y))?;
    let index = *view.shown().get(row + view.scroll)?;

    Some(match view.section {
        Section::Appearance => Target::Toggle(index),
        Section::Keys => Target::Binding(index),
    })
}
