use std::path::{Path, PathBuf};

use crc_config::keymap::{Chord, Key};
use crc_config::recent::{self, Recent};
use crc_config::{Binding, Keymap, Settings};

mod chords {
    use super::*;

    #[test]
    fn a_plain_letter_parses() {
        let chord = Chord::parse("s").expect("a chord");
        assert_eq!(chord.key, Key::Char('s'));
        assert!(!chord.ctrl && !chord.alt && !chord.shift);
    }

    #[test]
    fn modifiers_stack_in_any_order() {
        let one = Chord::parse("ctrl+shift+z").expect("a chord");
        let other = Chord::parse("shift+ctrl+z").expect("a chord");

        assert_eq!(one, other, "order of modifiers does not change the chord");
        assert!(one.ctrl && one.shift && !one.alt);
    }

    #[test]
    fn case_and_spaces_do_not_matter() {
        assert_eq!(Chord::parse("Ctrl+S"), Chord::parse(" ctrl + s "));
    }

    #[test]
    fn named_keys_are_understood() {
        assert_eq!(Chord::parse("escape").unwrap().key, Key::Escape);
        assert_eq!(Chord::parse("esc").unwrap().key, Key::Escape);
        assert_eq!(Chord::parse("pgdn").unwrap().key, Key::PageDown);
        assert_eq!(Chord::parse("ctrl+home").unwrap().key, Key::Home);
    }

    #[test]
    fn cmd_reads_as_ctrl_so_a_mac_keymap_still_loads() {
        assert_eq!(Chord::parse("cmd+s"), Chord::parse("ctrl+s"));
    }

    #[test]
    fn nonsense_is_rejected_rather_than_guessed() {
        assert_eq!(Chord::parse(""), None, "empty");
        assert_eq!(Chord::parse("ctrl"), None, "a modifier is not a chord");
        assert_eq!(Chord::parse("ctrl+"), None, "trailing plus");
        assert_eq!(Chord::parse("ctrl+ss"), None, "two letters is not a key");
        assert_eq!(Chord::parse("ctrl+a+b"), None, "two keys in one chord");
    }

    #[test]
    fn a_chord_prints_the_way_it_is_written() {
        assert_eq!(
            Chord::parse("ctrl+shift+z").unwrap().label(),
            "Ctrl+Shift+Z"
        );
        assert_eq!(Chord::parse("alt+1").unwrap().label(), "Alt+1");
        assert_eq!(Chord::parse("escape").unwrap().label(), "Esc");
    }
}

mod keymaps {
    use super::*;

    #[test]
    fn the_defaults_resolve() {
        let map = Keymap::default();

        assert_eq!(map.command(&Chord::parse("ctrl+s").unwrap()), Some("save"));
        assert_eq!(
            map.command(&Chord::parse("ctrl+k").unwrap()),
            Some("palette")
        );
        assert_eq!(map.command(&Chord::parse("alt+z").unwrap()), Some("zen"));
    }

    #[test]
    fn an_unbound_chord_resolves_to_nothing() {
        let map = Keymap::default();
        assert_eq!(map.command(&Chord::parse("ctrl+j").unwrap()), None);
    }

    #[test]
    fn a_later_binding_replaces_an_earlier_one_on_the_same_chord() {
        let (map, _) = Keymap::from_bindings(&[
            Binding::new("ctrl+s", "save"),
            Binding::new("ctrl+s", "sidebar"),
        ]);

        assert_eq!(
            map.command(&Chord::parse("ctrl+s").unwrap()),
            Some("sidebar")
        );
        assert_eq!(map.len(), 1, "the chord is bound once, not twice");
    }

    #[test]
    fn a_user_binding_wins_over_the_default() {
        let settings = Settings {
            keys: vec![Binding::new("ctrl+s", "palette")],
            ..Settings::default()
        };
        let (map, rejected) = settings.keymap();

        assert!(rejected.is_empty());
        assert_eq!(
            map.command(&Chord::parse("ctrl+s").unwrap()),
            Some("palette")
        );
        assert_eq!(
            map.command(&Chord::parse("ctrl+b").unwrap()),
            Some("sidebar"),
            "the rest of the defaults survive"
        );
    }

    #[test]
    fn an_empty_command_unbinds_the_chord() {
        let settings = Settings {
            keys: vec![Binding::new("ctrl+s", "")],
            ..Settings::default()
        };
        let (map, _) = settings.keymap();

        assert_eq!(map.command(&Chord::parse("ctrl+s").unwrap()), None);
    }

    #[test]
    fn a_broken_binding_is_reported_and_the_rest_still_load() {
        let settings = Settings {
            keys: vec![
                Binding::new("ctrl+", "save"),
                Binding::new("ctrl+j", "palette"),
            ],
            ..Settings::default()
        };
        let (map, rejected) = settings.keymap();

        assert_eq!(rejected, vec!["ctrl+".to_string()]);
        assert_eq!(
            map.command(&Chord::parse("ctrl+j").unwrap()),
            Some("palette")
        );
        assert_eq!(
            map.command(&Chord::parse("ctrl+s").unwrap()),
            Some("save"),
            "one bad line does not take the keymap down"
        );
    }

    #[test]
    fn a_command_can_report_the_keys_that_run_it() {
        let map = Keymap::default();
        assert_eq!(map.hint("save").as_deref(), Some("Ctrl+S"));
        assert_eq!(map.hint("never-bound"), None);
    }

    #[test]
    fn two_chords_may_share_one_command() {
        let map = Keymap::default();

        assert_eq!(map.command(&Chord::parse("ctrl+shift+z").unwrap()), Some("redo"));
        assert_eq!(map.command(&Chord::parse("ctrl+y").unwrap()), Some("redo"));
    }

    #[test]
    fn escape_does_not_close_the_editor() {
        let map = Keymap::default();

        assert_eq!(
            map.command(&Chord::parse("escape").unwrap()),
            None,
            "escape dismisses what is open, it never ends the session"
        );
    }
}

mod recents {
    use super::*;

    fn list(paths: &[&str]) -> Vec<Recent> {
        let mut out = Vec::new();
        for (index, path) in paths.iter().enumerate() {
            recent::remember(&mut out, Path::new(path), index as u64, 10);
        }
        out
    }

    #[test]
    fn the_newest_project_comes_first() {
        let recents = list(&["a", "b", "c"]);
        assert_eq!(
            recents.iter().map(|r| r.name.as_str()).collect::<Vec<_>>(),
            vec!["c", "b", "a"]
        );
    }

    #[test]
    fn reopening_moves_a_project_up_rather_than_duplicating_it() {
        let mut recents = list(&["a", "b", "c"]);
        recent::remember(&mut recents, Path::new("a"), 99, 10);

        assert_eq!(recents.len(), 3, "no duplicate entry");
        assert_eq!(recents[0].name, "a");
        assert_eq!(recents[0].opened_at, 99, "the time is refreshed");
    }

    #[test]
    fn the_list_is_capped() {
        let mut recents = Vec::new();
        for index in 0..30 {
            recent::remember(&mut recents, &PathBuf::from(index.to_string()), index, 5);
        }
        assert_eq!(recents.len(), 5);
        assert_eq!(recents[0].name, "29");
    }

    #[test]
    fn a_project_can_be_dropped_from_the_list() {
        let mut recents = list(&["a", "b"]);

        assert!(recent::forget(&mut recents, Path::new("a")));
        assert!(
            !recent::forget(&mut recents, Path::new("a")),
            "already gone"
        );
        assert_eq!(recents.len(), 1);
    }

    #[test]
    fn the_name_is_the_folder_not_the_whole_path() {
        let entry = Recent::new(PathBuf::from("d:/Project/CRC Code"), 0);
        assert_eq!(entry.name, "CRC Code");
    }

    #[test]
    fn elapsed_time_reads_in_words() {
        assert_eq!(recent::since(100, 130), "только что");
        assert_eq!(recent::since(0, 60 * 12), "12 мин назад");
        assert_eq!(recent::since(0, 3600 * 5), "5 ч назад");
        assert_eq!(recent::since(0, 86400 + 10), "вчера");
        assert_eq!(recent::since(0, 86400 * 4), "4 дн назад");
    }

    #[test]
    fn a_clock_that_went_backwards_does_not_panic() {
        assert_eq!(recent::since(500, 100), "только что");
    }
}

mod files {
    use super::*;

    #[test]
    fn settings_survive_a_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.toml");

        let mut settings = Settings {
            appearance: "light".to_string(),
            keys: vec![Binding::new("ctrl+p", "palette")],
            ..Settings::default()
        };
        settings.visible.minimap = false;
        settings.remember(Path::new("d:/Project/CRC Code"), 42);
        settings.save(&path).unwrap();

        let (loaded, error) = Settings::load(&path);

        assert!(error.is_none());
        assert_eq!(loaded, settings);
    }

    #[test]
    fn a_missing_file_gives_the_defaults_quietly() {
        let dir = tempfile::tempdir().unwrap();
        let (settings, error) = Settings::load(&dir.path().join("nothing.toml"));

        assert_eq!(settings, Settings::default());
        assert!(error.is_none(), "a first run is not an error");
    }

    #[test]
    fn a_broken_file_falls_back_to_defaults_and_says_why() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.toml");
        std::fs::write(&path, "this is not toml {{{").unwrap();

        let (settings, error) = Settings::load(&path);

        assert_eq!(settings, Settings::default());
        assert!(error.is_some(), "the reason is reported, not swallowed");
    }

    #[test]
    fn a_partial_file_keeps_the_defaults_for_everything_else() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.toml");
        std::fs::write(&path, "appearance = \"light\"\n").unwrap();

        let (settings, error) = Settings::load(&path);

        assert!(error.is_none());
        assert_eq!(settings.appearance, "light");
        assert_eq!(settings.density, Settings::default().density);
        assert!(
            settings.visible.minimap,
            "untouched flags keep their default"
        );
    }

    #[test]
    fn saving_creates_the_folder_it_needs() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("deep").join("nested").join("settings.toml");

        Settings::default().save(&path).unwrap();

        assert!(path.exists());
    }
}
