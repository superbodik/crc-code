use crc_config::Keymap;
use crc_config::keymap::{Chord, Key as Bound};
use crc_editor::Motion;
use winit::keyboard::{Key, ModifiersState, NamedKey};

pub enum Command {
    Quit,
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
    Undo,
    Redo,
    SelectAll,
    Save,
}

pub fn chord(key: &Key, modifiers: ModifiersState) -> Option<Chord> {
    let bound = match key {
        Key::Named(NamedKey::Escape) => Bound::Escape,
        Key::Named(NamedKey::Enter) => Bound::Enter,
        Key::Named(NamedKey::Tab) => Bound::Tab,
        Key::Named(NamedKey::Space) => Bound::Space,
        Key::Named(NamedKey::Backspace) => Bound::Backspace,
        Key::Named(NamedKey::Delete) => Bound::Delete,
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
            if chars.next().is_some() {
                return None;
            }
            Bound::Char(first.to_ascii_lowercase())
        }
        _ => return None,
    };

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
        "palette" => Command::OpenPalette,
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
    modifiers: ModifiersState,
    rows: usize,
    keymap: &Keymap,
) -> Option<Command> {
    if let Some(chord) = chord(key, modifiers)
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

        Key::Named(NamedKey::Backspace) => Some(Command::Backspace),
        Key::Named(NamedKey::Delete) => Some(Command::Delete),
        Key::Named(NamedKey::Enter) => Some(Command::Insert("\n".to_string())),
        Key::Named(NamedKey::Tab) => Some(Command::Insert("    ".to_string())),
        Key::Named(NamedKey::Space) if !control && !alt => Some(Command::Insert(" ".to_string())),

        Key::Character(text) if !control && !alt => Some(Command::Insert(text.to_string())),
        _ => None,
    }
}
