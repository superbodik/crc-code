use crc_config::Keymap;
use crc_config::keymap::{Chord, Key as Bound};
use crc_editor::Motion;
use winit::keyboard::{Key, KeyCode, ModifiersState, NamedKey, PhysicalKey};

pub enum Command {
    Quit,
    OpenSettings,
    OpenFolder,
    OpenFile,
    Find,
    FindStep { forward: bool },
    ShowWelcome,
    OpenPalette,
    CloseTab,
    ToggleZen,
    ToggleSidebar,
    ToggleAppearance,
    Density(u8),
    Move { motion: Motion, extend: bool },
    Insert(String),
    Backspace,
    Delete,
    DeleteWord { forward: bool },
    Copy,
    Cut,
    Paste,
    Undo,
    Redo,
    SelectAll,
    Save,
}

pub fn from_physical(key: PhysicalKey) -> Option<Bound> {
    let PhysicalKey::Code(code) = key else {
        return None;
    };

    Some(match code {
        KeyCode::KeyA => Bound::Char('a'),
        KeyCode::KeyB => Bound::Char('b'),
        KeyCode::KeyC => Bound::Char('c'),
        KeyCode::KeyD => Bound::Char('d'),
        KeyCode::KeyE => Bound::Char('e'),
        KeyCode::KeyF => Bound::Char('f'),
        KeyCode::KeyG => Bound::Char('g'),
        KeyCode::KeyH => Bound::Char('h'),
        KeyCode::KeyI => Bound::Char('i'),
        KeyCode::KeyJ => Bound::Char('j'),
        KeyCode::KeyK => Bound::Char('k'),
        KeyCode::KeyL => Bound::Char('l'),
        KeyCode::KeyM => Bound::Char('m'),
        KeyCode::KeyN => Bound::Char('n'),
        KeyCode::KeyO => Bound::Char('o'),
        KeyCode::KeyP => Bound::Char('p'),
        KeyCode::KeyQ => Bound::Char('q'),
        KeyCode::KeyR => Bound::Char('r'),
        KeyCode::KeyS => Bound::Char('s'),
        KeyCode::KeyT => Bound::Char('t'),
        KeyCode::KeyU => Bound::Char('u'),
        KeyCode::KeyV => Bound::Char('v'),
        KeyCode::KeyW => Bound::Char('w'),
        KeyCode::KeyX => Bound::Char('x'),
        KeyCode::KeyY => Bound::Char('y'),
        KeyCode::KeyZ => Bound::Char('z'),

        KeyCode::Digit0 | KeyCode::Numpad0 => Bound::Char('0'),
        KeyCode::Digit1 | KeyCode::Numpad1 => Bound::Char('1'),
        KeyCode::Digit2 | KeyCode::Numpad2 => Bound::Char('2'),
        KeyCode::Digit3 | KeyCode::Numpad3 => Bound::Char('3'),
        KeyCode::Digit4 | KeyCode::Numpad4 => Bound::Char('4'),
        KeyCode::Digit5 | KeyCode::Numpad5 => Bound::Char('5'),
        KeyCode::Digit6 | KeyCode::Numpad6 => Bound::Char('6'),
        KeyCode::Digit7 | KeyCode::Numpad7 => Bound::Char('7'),
        KeyCode::Digit8 | KeyCode::Numpad8 => Bound::Char('8'),
        KeyCode::Digit9 | KeyCode::Numpad9 => Bound::Char('9'),

        KeyCode::Comma => Bound::Char(','),
        KeyCode::Period => Bound::Char('.'),
        KeyCode::Slash => Bound::Char('/'),
        KeyCode::Backslash => Bound::Char('\\'),
        KeyCode::Semicolon => Bound::Char(';'),
        KeyCode::Quote => Bound::Char('\''),
        KeyCode::BracketLeft => Bound::Char('['),
        KeyCode::BracketRight => Bound::Char(']'),
        KeyCode::Minus => Bound::Char('-'),
        KeyCode::Equal => Bound::Char('='),
        KeyCode::Backquote => Bound::Char('`'),

        KeyCode::Escape => Bound::Escape,
        KeyCode::Enter | KeyCode::NumpadEnter => Bound::Enter,
        KeyCode::Tab => Bound::Tab,
        KeyCode::Space => Bound::Space,
        KeyCode::Backspace => Bound::Backspace,
        KeyCode::Delete => Bound::Delete,
        KeyCode::Insert => Bound::Insert,
        KeyCode::Home => Bound::Home,
        KeyCode::End => Bound::End,
        KeyCode::PageUp => Bound::PageUp,
        KeyCode::PageDown => Bound::PageDown,
        KeyCode::ArrowLeft => Bound::Left,
        KeyCode::ArrowRight => Bound::Right,
        KeyCode::ArrowUp => Bound::Up,
        KeyCode::ArrowDown => Bound::Down,

        _ => return None,
    })
}

pub fn from_logical(key: &Key) -> Option<Bound> {
    Some(match key {
        Key::Named(NamedKey::Escape) => Bound::Escape,
        Key::Named(NamedKey::Enter) => Bound::Enter,
        Key::Named(NamedKey::Tab) => Bound::Tab,
        Key::Named(NamedKey::Space) => Bound::Space,
        Key::Named(NamedKey::Backspace) => Bound::Backspace,
        Key::Named(NamedKey::Delete) => Bound::Delete,
        Key::Named(NamedKey::Insert) => Bound::Insert,
        Key::Named(NamedKey::Home) => Bound::Home,
        Key::Named(NamedKey::End) => Bound::End,
        Key::Named(NamedKey::PageUp) => Bound::PageUp,
        Key::Named(NamedKey::PageDown) => Bound::PageDown,
        Key::Named(NamedKey::ArrowLeft) => Bound::Left,
        Key::Named(NamedKey::ArrowRight) => Bound::Right,
        Key::Named(NamedKey::ArrowUp) => Bound::Up,
        Key::Named(NamedKey::ArrowDown) => Bound::Down,
        Key::Character(text) => {
            let mut chars = text.chars();
            let first = chars.next()?;
            if chars.next().is_some() || !first.is_ascii() {
                return None;
            }
            Bound::Char(first.to_ascii_lowercase())
        }
        _ => return None,
    })
}

pub fn chord(key: &Key, physical: PhysicalKey, modifiers: ModifiersState) -> Option<Chord> {
    let bound = from_physical(physical).or_else(|| from_logical(key))?;

    Some(Chord {
        key: bound,
        ctrl: modifiers.control_key(),
        alt: modifiers.alt_key(),
        shift: modifiers.shift_key(),
    })
}

pub fn command_named(name: &str) -> Option<Command> {
    Some(match name {
        "save" => Command::Save,
        "close-tab" => Command::CloseTab,
        "quit" => Command::Quit,
        "undo" => Command::Undo,
        "redo" => Command::Redo,
        "select-all" => Command::SelectAll,
        "copy" => Command::Copy,
        "cut" => Command::Cut,
        "paste" => Command::Paste,
        "delete-word-back" => Command::DeleteWord { forward: false },
        "delete-word-forward" => Command::DeleteWord { forward: true },
        "palette" => Command::OpenPalette,
        "open-folder" => Command::OpenFolder,
        "open-file" => Command::OpenFile,
        "find" => Command::Find,
        "find-next" => Command::FindStep { forward: true },
        "find-previous" => Command::FindStep { forward: false },
        "welcome" => Command::ShowWelcome,
        "settings" => Command::OpenSettings,
        "theme" => Command::ToggleAppearance,
        "sidebar" => Command::ToggleSidebar,
        "zen" => Command::ToggleZen,
        "calm" => Command::Density(1),
        "balanced" => Command::Density(2),
        "dense" => Command::Density(3),
        _ => return None,
    })
}

pub fn resolve(
    key: &Key,
    physical: PhysicalKey,
    modifiers: ModifiersState,
    rows: usize,
    keymap: &Keymap,
) -> Option<Command> {
    if let Some(chord) = chord(key, physical, modifiers)
        && let Some(name) = keymap.command(&chord)
        && let Some(command) = command_named(name)
    {
        return Some(command);
    }

    let control = modifiers.control_key();
    let shift = modifiers.shift_key();
    let alt = modifiers.alt_key();

    let motion = |motion: Motion| {
        Some(Command::Move {
            motion,
            extend: shift,
        })
    };

    match key {
        Key::Named(NamedKey::ArrowLeft) if control => motion(Motion::WordLeft),
        Key::Named(NamedKey::ArrowRight) if control => motion(Motion::WordRight),
        Key::Named(NamedKey::ArrowLeft) => motion(Motion::Left),
        Key::Named(NamedKey::ArrowRight) => motion(Motion::Right),
        Key::Named(NamedKey::ArrowUp) => motion(Motion::Up),
        Key::Named(NamedKey::ArrowDown) => motion(Motion::Down),
        Key::Named(NamedKey::PageUp) => motion(Motion::PageUp(rows.max(1))),
        Key::Named(NamedKey::PageDown) => motion(Motion::PageDown(rows.max(1))),
        Key::Named(NamedKey::Home) if control => motion(Motion::DocumentStart),
        Key::Named(NamedKey::End) if control => motion(Motion::DocumentEnd),
        Key::Named(NamedKey::Home) => motion(Motion::LineStart),
        Key::Named(NamedKey::End) => motion(Motion::LineEnd),

        Key::Named(NamedKey::Backspace) if control => Some(Command::DeleteWord { forward: false }),
        Key::Named(NamedKey::Delete) if control => Some(Command::DeleteWord { forward: true }),
        Key::Named(NamedKey::Backspace) => Some(Command::Backspace),
        Key::Named(NamedKey::Delete) => Some(Command::Delete),
        Key::Named(NamedKey::Enter) => Some(Command::Insert("\n".to_string())),
        Key::Named(NamedKey::Tab) => Some(Command::Insert("    ".to_string())),
        Key::Named(NamedKey::Space) if !control && !alt => Some(Command::Insert(" ".to_string())),

        Key::Character(text) if !control && !alt => Some(Command::Insert(text.to_string())),
        _ => None,
    }
}
