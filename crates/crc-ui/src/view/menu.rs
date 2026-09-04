use crate::geometry::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    NewFile,
    NewFolder,
    Rename,
    Delete,
    CopyPath,
    Reveal,
    Refresh,
}

impl MenuAction {
    pub const fn title(self) -> &'static str {
        match self {
            MenuAction::NewFile => "Новый файл",
            MenuAction::NewFolder => "Новая папка",
            MenuAction::Rename => "Переименовать",
            MenuAction::Delete => "Удалить",
            MenuAction::CopyPath => "Копировать путь",
            MenuAction::Reveal => "Показать в проводнике",
            MenuAction::Refresh => "Обновить дерево",
        }
    }

    pub const fn glyph(self) -> char {
        match self {
            MenuAction::NewFile => crate::icon::NEW_FILE,
            MenuAction::NewFolder => crate::icon::NEW_FOLDER,
            MenuAction::Rename => crate::icon::RENAME,
            MenuAction::Delete => crate::icon::DELETE,
            MenuAction::CopyPath => crate::icon::COPY_PATH,
            MenuAction::Reveal => crate::icon::REVEAL,
            MenuAction::Refresh => crate::icon::REFRESH,
        }
    }

    pub const fn destructive(self) -> bool {
        matches!(self, MenuAction::Delete)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuItem {
    Action(MenuAction),
    Separator,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MenuView {
    pub at: (f32, f32),
    pub items: Vec<MenuItem>,
    pub subject: Option<String>,
    pub hovered: Option<usize>,
}

impl MenuView {
    pub fn for_row(subject: impl Into<String>, is_dir: bool) -> Self {
        let mut items = vec![
            MenuItem::Action(MenuAction::NewFile),
            MenuItem::Action(MenuAction::NewFolder),
            MenuItem::Separator,
            MenuItem::Action(MenuAction::Rename),
            MenuItem::Action(MenuAction::Delete),
            MenuItem::Separator,
            MenuItem::Action(MenuAction::CopyPath),
            MenuItem::Action(MenuAction::Reveal),
        ];
        if is_dir {
            items.push(MenuItem::Separator);
            items.push(MenuItem::Action(MenuAction::Refresh));
        }

        Self {
            at: (0.0, 0.0),
            items,
            subject: Some(subject.into()),
            hovered: None,
        }
    }

    pub fn for_root() -> Self {
        Self {
            at: (0.0, 0.0),
            items: vec![
                MenuItem::Action(MenuAction::NewFile),
                MenuItem::Action(MenuAction::NewFolder),
                MenuItem::Separator,
                MenuItem::Action(MenuAction::Refresh),
            ],
            subject: None,
            hovered: None,
        }
    }

    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.at = (x, y);
        self
    }

    pub fn action(&self, index: usize) -> Option<MenuAction> {
        match self.items.get(index) {
            Some(MenuItem::Action(action)) => Some(*action),
            _ => None,
        }
    }
}

pub const WIDTH: f32 = 232.0;
pub const ROW: f32 = 30.0;
pub const SEPARATOR: f32 = 7.0;
pub const PADDING: f32 = 6.0;

#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    pub panel: Rect,
    pub rows: Vec<Rect>,
}

pub fn layout(window: Rect, view: &MenuView, scale: f32) -> Layout {
    let scale = scale.max(0.5);
    let width = WIDTH * scale;

    let height = view
        .items
        .iter()
        .map(|item| match item {
            MenuItem::Action(_) => ROW * scale,
            MenuItem::Separator => SEPARATOR * scale,
        })
        .sum::<f32>()
        + PADDING * 2.0 * scale;

    let x = view.at.0.min(window.right() - width).max(window.x);
    let y = view.at.1.min(window.bottom() - height).max(window.y);
    let panel = Rect::new(x, y, width, height);

    let mut rows = Vec::with_capacity(view.items.len());
    let mut top = panel.y + PADDING * scale;
    for item in &view.items {
        let tall = match item {
            MenuItem::Action(_) => ROW * scale,
            MenuItem::Separator => SEPARATOR * scale,
        };
        rows.push(Rect::new(panel.x, top, panel.width, tall));
        top += tall;
    }

    Layout { panel, rows }
}

pub fn item_at(layout: &Layout, view: &MenuView, x: f32, y: f32) -> Option<usize> {
    if !layout.panel.contains(x, y) {
        return None;
    }
    layout
        .rows
        .iter()
        .position(|rect| rect.contains(x, y))
        .filter(|index| view.action(*index).is_some())
}
