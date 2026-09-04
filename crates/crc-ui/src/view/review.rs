use serde_json::Value;

use crate::geometry::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Allow,
    Deny,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReviewView {
    pub tool: String,
    pub file: Option<String>,
    pub detail: Vec<String>,
    pub hovered: Option<Target>,
}

impl ReviewView {
    pub fn title(&self) -> String {
        match self.tool.as_str() {
            "Write" => "Записать файл".to_string(),
            "Edit" => "Изменить файл".to_string(),
            "NotebookEdit" => "Изменить блокнот".to_string(),
            "Bash" | "PowerShell" => "Выполнить команду".to_string(),
            other => format!("Разрешить {other}"),
        }
    }

    pub fn subject(&self) -> String {
        match self.file.as_ref() {
            Some(file) => tail(file, 82),
            None => self.tool.clone(),
        }
    }
}

pub fn describe(input: &Value) -> Vec<String> {
    let Some(fields) = input.as_object() else {
        return Vec::new();
    };

    let mut lines = Vec::new();
    for key in [
        "command",
        "old_string",
        "new_string",
        "content",
        "pattern",
        "prompt",
        "url",
    ] {
        let Some(found) = fields.get(key).and_then(Value::as_str) else {
            continue;
        };
        if found.is_empty() {
            continue;
        }

        lines.push(format!("{key}:"));
        for line in found.lines().take(12) {
            lines.push(format!("  {}", cut(line, 90)));
        }
        if found.lines().count() > 12 {
            lines.push("  ...".to_string());
        }
    }

    lines.truncate(40);
    lines
}

pub fn tail(path: &str, limit: usize) -> String {
    let letters = path.chars().count();
    if letters <= limit {
        return path.to_string();
    }

    let kept: String = path
        .chars()
        .skip(letters - limit + 3)
        .collect();
    format!("...{kept}")
}

fn cut(line: &str, limit: usize) -> String {
    if line.chars().count() <= limit {
        return line.to_string();
    }
    let kept: String = line.chars().take(limit).collect();
    format!("{kept}...")
}

pub const WIDTH: f32 = 720.0;
pub const HEADER: f32 = 58.0;
pub const FOOTER: f32 = 62.0;
pub const ROW: f32 = 18.0;
pub const PADDING: f32 = 20.0;
pub const BUTTON: f32 = 36.0;

#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    pub panel: Rect,
    pub title: Rect,
    pub subject: Rect,
    pub body: Rect,
    pub rows: Vec<Rect>,
    pub allow: Rect,
    pub deny: Rect,
}

pub fn layout(window: Rect, view: &ReviewView, scale: f32) -> Layout {
    let scale = scale.max(0.5);
    let width = (WIDTH * scale).min(window.width - 60.0 * scale);

    let lines = view.detail.len().min(20) as f32;
    let height = ((HEADER + FOOTER) * scale + lines * ROW * scale + PADDING * scale)
        .min(window.height - 60.0 * scale);

    let panel = Rect::new(
        window.x + (window.width - width) / 2.0,
        window.y + (window.height - height) / 3.0,
        width,
        height,
    );

    let inset = PADDING * scale;
    let title = Rect::new(panel.x + inset, panel.y + inset, panel.width - inset * 2.0, 22.0 * scale);
    let subject = Rect::new(title.x, title.bottom() + 2.0 * scale, title.width, 18.0 * scale);

    let body = Rect::new(
        title.x,
        subject.bottom() + 8.0 * scale,
        title.width,
        (panel.bottom() - FOOTER * scale - subject.bottom() - 8.0 * scale).max(0.0),
    );

    let fits = (body.height / (ROW * scale)).floor().max(0.0) as usize;
    let mut rows = Vec::new();
    let mut y = body.y;
    for _ in 0..view.detail.len().min(fits) {
        rows.push(Rect::new(body.x, y, body.width, ROW * scale));
        y += ROW * scale;
    }

    let button = BUTTON * scale;
    let allow_width = 148.0 * scale;
    let deny_width = 128.0 * scale;

    let allow = Rect::new(
        panel.right() - inset - allow_width,
        panel.bottom() - inset - button,
        allow_width,
        button,
    );
    let deny = Rect::new(
        allow.x - 8.0 * scale - deny_width,
        allow.y,
        deny_width,
        button,
    );

    Layout {
        panel,
        title,
        subject,
        body,
        rows,
        allow,
        deny,
    }
}

pub fn target_at(layout: &Layout, x: f32, y: f32) -> Option<Target> {
    if layout.allow.contains(x, y) {
        return Some(Target::Allow);
    }
    if layout.deny.contains(x, y) {
        return Some(Target::Deny);
    }
    None
}
