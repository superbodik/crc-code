use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Chord {
    pub key: Key,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Key {
    #[default]
    None,
    Char(char),
    Escape,
    Enter,
    Tab,
    Space,
    Backspace,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    Left,
    Right,
    Up,
    Down,
}

impl Key {
    pub fn named(name: &str) -> Option<Key> {
        Some(match name {
            "escape" | "esc" => Key::Escape,
            "enter" | "return" => Key::Enter,
            "tab" => Key::Tab,
            "space" => Key::Space,
            "backspace" => Key::Backspace,
            "delete" | "del" => Key::Delete,
            "insert" | "ins" => Key::Insert,
            "home" => Key::Home,
            "end" => Key::End,
            "pageup" | "pgup" => Key::PageUp,
            "pagedown" | "pgdn" => Key::PageDown,
            "left" => Key::Left,
            "right" => Key::Right,
            "up" => Key::Up,
            "down" => Key::Down,
            _ => return None,
        })
    }

    pub fn label(self) -> String {
        match self {
            Key::None => String::new(),
            Key::Char(c) => c.to_uppercase().to_string(),
            Key::Escape => "Esc".into(),
            Key::Enter => "Enter".into(),
            Key::Tab => "Tab".into(),
            Key::Space => "Space".into(),
            Key::Backspace => "Backspace".into(),
            Key::Delete => "Delete".into(),
            Key::Insert => "Insert".into(),
            Key::Home => "Home".into(),
            Key::End => "End".into(),
            Key::PageUp => "PgUp".into(),
            Key::PageDown => "PgDn".into(),
            Key::Left => "Left".into(),
            Key::Right => "Right".into(),
            Key::Up => "Up".into(),
            Key::Down => "Down".into(),
        }
    }
}

impl Chord {
    pub fn parse(spec: &str) -> Option<Chord> {
        let mut chord = Chord::default();
        let spec = spec.trim();
        if spec.is_empty() {
            return None;
        }

        for part in spec.split('+') {
            let part = part.trim().to_lowercase();
            match part.as_str() {
                "" => return None,
                "ctrl" | "control" | "cmd" | "meta" => chord.ctrl = true,
                "alt" | "option" => chord.alt = true,
                "shift" => chord.shift = true,
                other => {
                    if chord.key != Key::None {
                        return None;
                    }
                    chord.key = match Key::named(other) {
                        Some(key) => key,
                        None => {
                            let mut chars = other.chars();
                            let first = chars.next()?;
                            if chars.next().is_some() || !first.is_ascii() {
                                return None;
                            }
                            Key::Char(first.to_ascii_lowercase())
                        }
                    };
                }
            }
        }

        (chord.key != Key::None).then_some(chord)
    }

    pub fn label(&self) -> String {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl".to_string());
        }
        if self.alt {
            parts.push("Alt".to_string());
        }
        if self.shift {
            parts.push("Shift".to_string());
        }
        parts.push(self.key.label());
        parts.join("+")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Binding {
    pub keys: String,
    pub command: String,
}

impl Binding {
    pub fn new(keys: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            keys: keys.into(),
            command: command.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keymap {
    bindings: Vec<(Chord, String)>,
}

impl Keymap {
    pub fn from_bindings(bindings: &[Binding]) -> (Keymap, Vec<String>) {
        let mut parsed = Vec::new();
        let mut rejected = Vec::new();

        for binding in bindings {
            match Chord::parse(&binding.keys) {
                Some(chord) => {
                    parsed.retain(|(existing, _)| existing != &chord);
                    if !binding.command.is_empty() {
                        parsed.push((chord, binding.command.clone()));
                    }
                }
                None => rejected.push(binding.keys.clone()),
            }
        }

        (Keymap { bindings: parsed }, rejected)
    }

    pub fn command(&self, chord: &Chord) -> Option<&str> {
        self.bindings
            .iter()
            .rev()
            .find(|(bound, _)| bound == chord)
            .map(|(_, command)| command.as_str())
    }

    pub fn chord_for(&self, command: &str) -> Option<Chord> {
        self.bindings
            .iter()
            .find(|(_, bound)| bound == command)
            .map(|(chord, _)| *chord)
    }

    pub fn hint(&self, command: &str) -> Option<String> {
        self.chord_for(command).map(|chord| chord.label())
    }

    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Chord, &str)> {
        self.bindings
            .iter()
            .map(|(chord, command)| (chord, command.as_str()))
    }
}

impl Default for Keymap {
    fn default() -> Self {
        Keymap::from_bindings(&defaults()).0
    }
}

pub fn defaults() -> Vec<Binding> {
    vec![
        Binding::new("ctrl+s", "save"),
        Binding::new("ctrl+w", "close-tab"),
        Binding::new("ctrl+q", "quit"),
        Binding::new("ctrl+z", "undo"),
        Binding::new("ctrl+shift+z", "redo"),
        Binding::new("ctrl+y", "redo"),
        Binding::new("ctrl+a", "select-all"),
        Binding::new("ctrl+c", "copy"),
        Binding::new("ctrl+x", "cut"),
        Binding::new("ctrl+v", "paste"),
        Binding::new("ctrl+insert", "copy"),
        Binding::new("shift+insert", "paste"),
        Binding::new("ctrl+backspace", "delete-word-back"),
        Binding::new("ctrl+delete", "delete-word-forward"),
        Binding::new("ctrl+k", "palette"),
        Binding::new("ctrl+o", "open-folder"),
        Binding::new("ctrl+shift+o", "open-file"),
        Binding::new("ctrl+,", "settings"),
        Binding::new("ctrl+shift+,", "settings"),
        Binding::new("ctrl+shift+w", "welcome"),
        Binding::new("ctrl+d", "theme"),
        Binding::new("ctrl+b", "sidebar"),
        Binding::new("alt+z", "zen"),
        Binding::new("alt+1", "calm"),
        Binding::new("alt+2", "balanced"),
        Binding::new("alt+3", "dense"),
    ]
}
