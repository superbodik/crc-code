use crate::event::Event;

pub fn moves(count: u64) -> &'static str {
    let last_two = count % 100;
    if (11..=14).contains(&last_two) {
        return "ходов";
    }
    match count % 10 {
        1 => "ход",
        2..=4 => "хода",
        _ => "ходов",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Speaker {
    You,
    Claude,
    Tool,
    Editor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Turn {
    pub speaker: Speaker,
    pub text: String,
}

impl Turn {
    pub fn new(speaker: Speaker, text: impl Into<String>) -> Self {
        Self {
            speaker,
            text: text.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Talk {
    pub turns: Vec<Turn>,
    pub session: String,
    pub model: String,
    pub cost: f64,
    pub busy: bool,
    pub alive: bool,
    pub note: Option<String>,
}

impl Talk {
    pub fn asked(&mut self, text: impl Into<String>) {
        self.turns.push(Turn::new(Speaker::You, text));
        self.busy = true;
        self.note = None;
    }

    pub fn take(&mut self, event: Event) {
        match event {
            Event::Ready {
                session,
                model,
                tools,
            } => {
                self.session = session;
                self.model = model;
                self.alive = true;
                self.note = Some(format!("{} инструментов наготове", tools));
            }
            Event::Said(text) => self.turns.push(Turn::new(Speaker::Claude, text)),
            Event::Thought(_) => {}
            Event::Using { tool, detail } => {
                let text = if detail.is_empty() {
                    tool
                } else {
                    format!("{tool}: {detail}")
                };
                self.turns.push(Turn::new(Speaker::Tool, text));
            }
            Event::Returned { trouble, .. } => {
                if trouble
                    && let Some(last) = self.turns.last_mut()
                    && last.speaker == Speaker::Tool
                {
                    last.text.push_str(" — не вышло");
                }
            }
            Event::Finished { cost, turns, .. } => {
                self.cost += cost;
                self.busy = false;
                self.note = Some(format!(
                    "{turns} {}, {:.2} доллара всего",
                    moves(turns),
                    self.cost
                ));
            }
            Event::Limited { status, .. } => {
                if status != "allowed" {
                    self.note = Some(format!("лимит: {status}"));
                }
            }
            Event::Trouble(text) => {
                self.busy = false;
                self.turns.push(Turn::new(Speaker::Editor, text));
            }
            Event::Gone => {
                self.busy = false;
                self.alive = false;
                self.note = Some("агент закрылся".to_string());
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.turns.is_empty()
    }
}
