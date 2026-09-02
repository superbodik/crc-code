use crc_editor::Motion;
use winit::keyboard::{Key, ModifiersState, NamedKey};

pub enum Command {
    Quit,
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

pub fn resolve(key: &Key, modifiers: ModifiersState, rows: usize) -> Option<Command> {
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
        Key::Named(NamedKey::Escape) => Some(Command::Quit),

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

        Key::Character(text) if control => match text.to_lowercase().as_str() {
            "s" | "ы" => Some(Command::Save),
            "z" | "я" if shift => Some(Command::Redo),
            "z" | "я" => Some(Command::Undo),
            "y" | "н" => Some(Command::Redo),
            "a" | "ф" => Some(Command::SelectAll),
            "b" | "и" => Some(Command::ToggleSidebar),
            "d" | "в" => Some(Command::ToggleAppearance),
            "q" | "й" => Some(Command::Quit),
            _ => None,
        },

        Key::Character(text) if alt => match text.to_lowercase().as_str() {
            "z" | "я" => Some(Command::ToggleZen),
            _ => None,
        },

        Key::Character(text) => Some(Command::Insert(text.to_string())),

        _ => None,
    }
}

pub fn density_shortcut(key: &Key, modifiers: ModifiersState) -> Option<Command> {
    if !modifiers.alt_key() {
        return None;
    }
    let Key::Character(text) = key else {
        return None;
    };
    match text.as_str() {
        "1" => Some(Command::Density(1)),
        "2" => Some(Command::Density(2)),
        "3" => Some(Command::Density(3)),
        _ => None,
    }
}
