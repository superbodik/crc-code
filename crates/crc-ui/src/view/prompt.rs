use crate::geometry::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptKind {
    NewFile,
    NewFolder,
    Rename,
    Delete,
}

impl PromptKind {
    pub const fn title(self) -> &'static str {
        match self {
            PromptKind::NewFile => "Новый файл",
            PromptKind::NewFolder => "Новая папка",
            PromptKind::Rename => "Переименовать",
            PromptKind::Delete => "Удалить",
        }
    }

    pub const fn confirm(self) -> &'static str {
        match self {
            PromptKind::NewFile | PromptKind::NewFolder => "Создать",
            PromptKind::Rename => "Переименовать",
            PromptKind::Delete => "Удалить",
        }
    }

    pub const fn asks_for_a_name(self) -> bool {
        !matches!(self, PromptKind::Delete)
    }

    pub const fn destructive(self) -> bool {
        matches!(self, PromptKind::Delete)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Field,
    Confirm,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptView {
    pub kind: PromptKind,
    pub value: String,
    pub note: String,
    pub complaint: Option<String>,
    pub hovered: Option<Target>,
}

impl PromptView {
    pub fn new(kind: PromptKind, note: impl Into<String>) -> Self {
        Self {
            kind,
            value: String::new(),
            note: note.into(),
            complaint: None,
            hovered: None,
        }
    }

    pub fn seeded(kind: PromptKind, note: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            ..Self::new(kind, note)
        }
    }

    pub fn trimmed(&self) -> &str {
        self.value.trim()
    }

    pub fn ready(&self) -> bool {
        if !self.kind.asks_for_a_name() {
            return true;
        }
        let name = self.trimmed();
        !name.is_empty()
            && !name.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|'])
            && name != "."
            && name != ".."
    }
}

pub const WIDTH: f32 = 420.0;
pub const HEIGHT: f32 = 178.0;
pub const FIELD: f32 = 38.0;
pub const BUTTON: f32 = 34.0;
pub const PADDING: f32 = 20.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    pub panel: Rect,
    pub title: Rect,
    pub note: Rect,
    pub field: Rect,
    pub confirm: Rect,
    pub cancel: Rect,
}

pub fn layout(window: Rect, view: &PromptView, scale: f32) -> Layout {
    let scale = scale.max(0.5);
    let width = (WIDTH * scale).min(window.width - 40.0 * scale);
    let height = if view.kind.asks_for_a_name() {
        HEIGHT * scale
    } else {
        (HEIGHT - FIELD - 12.0) * scale
    };

    let panel = Rect::new(
        window.x + (window.width - width) / 2.0,
        window.y + (window.height - height) / 3.0,
        width,
        height,
    );

    let inset = PADDING * scale;
    let title = Rect::new(panel.x + inset, panel.y + inset, panel.width - inset * 2.0, 24.0 * scale);
    let note = Rect::new(title.x, title.bottom() + 2.0 * scale, title.width, 20.0 * scale);

    let field = if view.kind.asks_for_a_name() {
        Rect::new(title.x, note.bottom() + 10.0 * scale, title.width, FIELD * scale)
    } else {
        Rect::new(title.x, note.bottom(), title.width, 0.0)
    };

    let button = BUTTON * scale;
    let confirm_width = 128.0 * scale;
    let cancel_width = 104.0 * scale;

    let confirm = Rect::new(
        panel.right() - inset - confirm_width,
        panel.bottom() - inset - button,
        confirm_width,
        button,
    );
    let cancel = Rect::new(
        confirm.x - 8.0 * scale - cancel_width,
        confirm.y,
        cancel_width,
        button,
    );

    Layout {
        panel,
        title,
        note,
        field,
        confirm,
        cancel,
    }
}

pub fn target_at(layout: &Layout, x: f32, y: f32) -> Option<Target> {
    if !layout.panel.contains(x, y) {
        return None;
    }
    if layout.confirm.contains(x, y) {
        return Some(Target::Confirm);
    }
    if layout.cancel.contains(x, y) {
        return Some(Target::Cancel);
    }
    if layout.field.height > 0.0 && layout.field.contains(x, y) {
        return Some(Target::Field);
    }
    None
}
