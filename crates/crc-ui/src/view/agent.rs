use crc_agent::{Speaker, Talk};

use crate::geometry::Rect;
use crate::icon;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Composer,
    Send,
    Stop,
    Close,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AgentView {
    pub talk: Talk,
    pub draft: String,
    pub focused: bool,
    pub scroll: usize,
    pub hovered: Option<Target>,
    pub missing: bool,
    pub context: Option<String>,
}

impl AgentView {
    pub fn stoppable(&self) -> bool {
        self.talk.busy && !self.missing
    }

    pub fn context_note(&self) -> Option<String> {
        let file = self.context.as_ref()?;
        if self.talk.busy || self.missing {
            return None;
        }
        Some(format!("в работе: {file}"))
    }

    pub fn ready_to_send(&self) -> bool {
        !self.draft.trim().is_empty() && !self.talk.busy && !self.missing
    }

    pub fn status(&self) -> String {
        if self.missing {
            return "claude не найден в PATH".to_string();
        }
        if self.talk.busy {
            return "думает...".to_string();
        }
        match self.talk.note.as_deref() {
            Some(note) => note.to_string(),
            None if self.talk.model.is_empty() => "готов к работе".to_string(),
            None => self.talk.model.clone(),
        }
    }

    pub fn greeting(&self) -> &'static str {
        if self.missing {
            "Поставь Claude Code и перезапусти редактор"
        } else {
            "Спроси что угодно про этот проект"
        }
    }
}

pub fn glyph(speaker: Speaker) -> char {
    match speaker {
        Speaker::You => icon::CHEVRON_RIGHT,
        Speaker::Claude => icon::ROBOT,
        Speaker::Tool => icon::TERMINAL,
        Speaker::Editor => icon::WARNING,
        Speaker::Note => icon::CHECK,
    }
}

pub const HEADER: f32 = 38.0;
pub const COMPOSER: f32 = 76.0;
pub const STATUS: f32 = 22.0;
pub const PADDING: f32 = 12.0;
pub const BUTTON: f32 = 26.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    pub header: Rect,
    pub close: Rect,
    pub transcript: Rect,
    pub status: Rect,
    pub composer: Rect,
    pub send: Rect,
}

pub fn layout(aside: Rect, scale: f32) -> Layout {
    let scale = scale.max(0.5);

    let header = Rect::new(aside.x, aside.y, aside.width, HEADER * scale);
    let close = Rect::new(
        header.right() - PADDING * scale - BUTTON * scale,
        header.y + (header.height - BUTTON * scale) / 2.0,
        BUTTON * scale,
        BUTTON * scale,
    );

    let composer_height = COMPOSER * scale;
    let status_height = STATUS * scale;

    let composer = Rect::new(
        aside.x + PADDING * scale,
        aside.bottom() - composer_height - PADDING * scale,
        (aside.width - PADDING * 2.0 * scale).max(0.0),
        composer_height,
    );
    let send = Rect::new(
        composer.right() - PADDING * scale - BUTTON * scale,
        composer.bottom() - PADDING * scale * 0.5 - BUTTON * scale,
        BUTTON * scale,
        BUTTON * scale,
    );
    let status = Rect::new(
        aside.x + PADDING * scale,
        composer.y - status_height,
        composer.width,
        status_height,
    );
    let transcript = Rect::new(
        aside.x,
        header.bottom(),
        aside.width,
        (status.y - header.bottom()).max(0.0),
    );

    Layout {
        header,
        close,
        transcript,
        status,
        composer,
        send,
    }
}

pub fn target_at(layout: &Layout, view: &AgentView, x: f32, y: f32) -> Option<Target> {
    if layout.close.contains(x, y) {
        return Some(Target::Close);
    }
    if layout.send.contains(x, y) {
        return Some(if view.stoppable() {
            Target::Stop
        } else {
            Target::Send
        });
    }
    if layout.composer.contains(x, y) {
        return Some(Target::Composer);
    }
    None
}

pub fn wrap(text: &str, columns: usize) -> Vec<String> {
    if columns == 0 {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        if paragraph.chars().count() <= columns {
            lines.push(paragraph.to_string());
            continue;
        }

        let mut line = String::new();
        for word in paragraph.split(' ') {
            let wanted = if line.is_empty() {
                word.chars().count()
            } else {
                line.chars().count() + 1 + word.chars().count()
            };

            if wanted > columns && !line.is_empty() {
                lines.push(std::mem::take(&mut line));
            }

            if word.chars().count() > columns {
                let mut rest: &str = word;
                while rest.chars().count() > columns {
                    let cut = rest
                        .char_indices()
                        .nth(columns)
                        .map(|(index, _)| index)
                        .unwrap_or(rest.len());
                    lines.push(rest[..cut].to_string());
                    rest = &rest[cut..];
                }
                line = rest.to_string();
                continue;
            }

            if line.is_empty() {
                line = word.to_string();
            } else {
                line.push(' ');
                line.push_str(word);
            }
        }
        lines.push(line);
    }

    lines
}
