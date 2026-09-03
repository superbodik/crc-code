use std::ops::Range;

use crate::geometry::Rect;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    pub id: &'static str,
    pub title: String,
    pub group: &'static str,
    pub hint: Option<String>,
}

impl Action {
    pub fn new(id: &'static str, title: impl Into<String>, group: &'static str) -> Self {
        Self {
            id,
            title: title.into(),
            group,
            hint: None,
        }
    }

    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub id: &'static str,
    pub title: String,
    pub group: &'static str,
    pub hint: Option<String>,
    pub matched: Vec<Range<usize>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PaletteView {
    pub query: String,
    pub rows: Vec<Row>,
    pub selected: usize,
}

impl PaletteView {
    pub fn selected_id(&self) -> Option<&'static str> {
        self.rows.get(self.selected).map(|row| row.id)
    }

    pub fn move_selection(&mut self, delta: isize) {
        if self.rows.is_empty() {
            self.selected = 0;
            return;
        }
        let last = self.rows.len() as isize - 1;
        let next = self.selected as isize + delta;
        self.selected = next.clamp(0, last) as usize;
    }
}

pub const MAX_ROWS: usize = 8;

pub fn filter(actions: &[Action], query: &str) -> Vec<Row> {
    let needle = query.trim().to_lowercase();

    let mut scored: Vec<(i32, Row)> = actions
        .iter()
        .filter_map(|action| {
            let (score, matched) = if needle.is_empty() {
                (0, Vec::new())
            } else {
                score(&action.title, &needle)?
            };
            Some((
                score,
                Row {
                    id: action.id,
                    title: action.title.clone(),
                    group: action.group,
                    hint: action.hint.clone(),
                    matched,
                },
            ))
        })
        .collect();

    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.title.cmp(&b.1.title)));
    scored
        .into_iter()
        .take(MAX_ROWS)
        .map(|(_, row)| row)
        .collect()
}

fn score(title: &str, needle: &str) -> Option<(i32, Vec<Range<usize>>)> {
    let haystack = title.to_lowercase();
    let mut matched: Vec<Range<usize>> = Vec::new();
    let mut score = 0;
    let mut previous_end: Option<usize> = None;
    let mut wanted = needle.chars().peekable();
    let mut target = wanted.next()?;

    for (index, character) in haystack.char_indices() {
        if character != target {
            continue;
        }
        let end = index + character.len_utf8();
        score += 1;

        if index == 0 {
            score += 6;
        } else if haystack[..index].ends_with([' ', '-', '_', ':']) {
            score += 4;
        }

        match matched.last_mut() {
            Some(last) if previous_end == Some(index) => {
                last.end = end;
                score += 4;
            }
            _ => matched.push(index..end),
        }
        previous_end = Some(end);

        match wanted.next() {
            Some(next) => target = next,
            None => {
                score -= (haystack.chars().count() as i32) / 8;
                return Some((score, matched));
            }
        }
    }
    None
}

pub const WIDTH: f32 = 620.0;
pub const TOP: f32 = 96.0;
pub const INPUT_HEIGHT: f32 = 52.0;
pub const ROW_HEIGHT: f32 = 38.0;
pub const FOOTER_HEIGHT: f32 = 32.0;
pub const PADDING: f32 = 8.0;

pub fn frame(window: Rect, rows: usize, scale: f32) -> Rect {
    let scale = scale.max(0.5);
    let width = (WIDTH * scale).min(window.width - 48.0 * scale);
    let height = (INPUT_HEIGHT + FOOTER_HEIGHT) * scale
        + rows.min(MAX_ROWS) as f32 * ROW_HEIGHT * scale
        + if rows == 0 { 0.0 } else { PADDING * scale };

    Rect::new(
        window.x + (window.width - width) / 2.0,
        window.y + TOP * scale,
        width,
        height.min(window.height - TOP * scale - 24.0 * scale),
    )
}

pub fn input_rect(frame: Rect, scale: f32) -> Rect {
    Rect::new(frame.x, frame.y, frame.width, INPUT_HEIGHT * scale.max(0.5))
}

pub fn row_rect(frame: Rect, index: usize, scale: f32) -> Rect {
    let scale = scale.max(0.5);
    Rect::new(
        frame.x,
        frame.y + INPUT_HEIGHT * scale + PADDING * scale + index as f32 * ROW_HEIGHT * scale,
        frame.width,
        ROW_HEIGHT * scale,
    )
}

pub fn footer_rect(frame: Rect, scale: f32) -> Rect {
    let height = FOOTER_HEIGHT * scale.max(0.5);
    Rect::new(frame.x, frame.bottom() - height, frame.width, height)
}

pub fn row_at(frame: Rect, rows: usize, scale: f32, x: f32, y: f32) -> Option<usize> {
    (0..rows.min(MAX_ROWS)).find(|index| row_rect(frame, *index, scale).contains(x, y))
}
