use crc_app::input::{Command, chord, resolve};
use crc_config::keymap::{Chord, Key as Bound, Keymap};
use winit::keyboard::{Key, KeyCode, ModifiersState, NamedKey, PhysicalKey, SmolStr};

fn latin(character: &str) -> Key {
    Key::Character(SmolStr::new(character))
}

fn code(code: KeyCode) -> PhysicalKey {
    PhysicalKey::Code(code)
}

fn ctrl() -> ModifiersState {
    ModifiersState::CONTROL
}

fn command(key: Key, physical: PhysicalKey, modifiers: ModifiersState) -> Option<Command> {
    resolve(&key, physical, modifiers, 40, &Keymap::default())
}

mod any_layout {
    use super::*;

    #[test]
    fn undo_answers_to_the_key_where_z_lives_whatever_it_types() {
        let russian = command(latin("я"), code(KeyCode::KeyZ), ctrl());
        assert!(matches!(russian, Some(Command::Undo)), "Ctrl+Z on Russian");

        let english = command(latin("z"), code(KeyCode::KeyZ), ctrl());
        assert!(matches!(english, Some(Command::Undo)), "Ctrl+Z on English");
    }

    #[test]
    fn copy_cut_and_paste_sit_on_their_keys_not_their_letters() {
        assert!(matches!(
            command(latin("с"), code(KeyCode::KeyC), ctrl()),
            Some(Command::Copy)
        ));
        assert!(matches!(
            command(latin("ч"), code(KeyCode::KeyX), ctrl()),
            Some(Command::Cut)
        ));
        assert!(matches!(
            command(latin("м"), code(KeyCode::KeyV), ctrl()),
            Some(Command::Paste)
        ));
    }

    #[test]
    fn the_palette_and_saving_survive_the_layout_too() {
        assert!(matches!(
            command(latin("л"), code(KeyCode::KeyK), ctrl()),
            Some(Command::OpenPalette)
        ));
        assert!(matches!(
            command(latin("ы"), code(KeyCode::KeyS), ctrl()),
            Some(Command::Save)
        ));
    }

    #[test]
    fn settings_answer_to_the_comma_key_position() {
        assert!(matches!(
            command(latin("б"), code(KeyCode::Comma), ctrl()),
            Some(Command::OpenSettings)
        ));
    }

    #[test]
    fn a_letter_typed_without_modifiers_is_the_letter_you_pressed() {
        let typed = command(latin("я"), code(KeyCode::KeyZ), ModifiersState::empty());

        match typed {
            Some(Command::Insert(text)) => assert_eq!(text, "я", "the layout decides the text"),
            _ => panic!("plain typing should insert, not run a command"),
        }
    }

    #[test]
    fn a_key_the_physical_map_does_not_know_falls_back_to_what_it_types() {
        let chord = chord(&latin("z"), PhysicalKey::Unidentified(
            winit::keyboard::NativeKeyCode::Unidentified,
        ), ctrl());

        assert_eq!(
            chord,
            Some(Chord {
                key: Bound::Char('z'),
                ctrl: true,
                alt: false,
                shift: false,
            })
        );
    }
}

mod whole_words {
    use super::*;

    #[test]
    fn ctrl_backspace_takes_a_word_and_plain_backspace_takes_a_character() {
        assert!(matches!(
            command(
                Key::Named(NamedKey::Backspace),
                code(KeyCode::Backspace),
                ctrl()
            ),
            Some(Command::DeleteWord { forward: false })
        ));
        assert!(matches!(
            command(
                Key::Named(NamedKey::Backspace),
                code(KeyCode::Backspace),
                ModifiersState::empty()
            ),
            Some(Command::Backspace)
        ));
    }

    #[test]
    fn ctrl_delete_reaches_forward() {
        assert!(matches!(
            command(Key::Named(NamedKey::Delete), code(KeyCode::Delete), ctrl()),
            Some(Command::DeleteWord { forward: true })
        ));
    }
}

mod windows_habits {
    use super::*;

    #[test]
    fn the_insert_pair_copies_and_pastes_as_windows_users_expect() {
        assert!(matches!(
            command(Key::Named(NamedKey::Insert), code(KeyCode::Insert), ctrl()),
            Some(Command::Copy)
        ));
        assert!(matches!(
            command(
                Key::Named(NamedKey::Insert),
                code(KeyCode::Insert),
                ModifiersState::SHIFT
            ),
            Some(Command::Paste)
        ));
    }

    #[test]
    fn escape_alone_runs_nothing() {
        assert!(
            command(
                Key::Named(NamedKey::Escape),
                code(KeyCode::Escape),
                ModifiersState::empty()
            )
            .is_none(),
            "escape is for dismissing overlays, not for the editor"
        );
    }
}
