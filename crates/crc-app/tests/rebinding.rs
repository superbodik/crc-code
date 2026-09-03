use crc_app::keys::overrides;
use crc_config::keymap::{Chord, Keymap};
use crc_config::{Binding, Settings};
use crc_ui::view::settings::BindingRow;

fn row(command: &str, keys: &str, changed: bool) -> BindingRow {
    BindingRow {
        command: command.to_string(),
        title: command.to_string(),
        keys: keys.to_string(),
        clash: None,
        changed,
    }
}

fn factory_rows() -> Vec<BindingRow> {
    let (factory, _) = Keymap::from_bindings(&crc_config::keymap::defaults());
    [
        "save",
        "close-tab",
        "palette",
        "open-folder",
        "settings",
        "theme",
        "sidebar",
        "zen",
        "undo",
        "redo",
        "copy",
        "paste",
    ]
    .into_iter()
    .map(|command| {
        row(
            command,
            &factory.hint(command).unwrap_or_default(),
            false,
        )
    })
    .collect()
}

fn keymap_from(rows: &[BindingRow]) -> Keymap {
    let settings = Settings {
        keys: overrides(rows),
        ..Settings::default()
    };
    settings.keymap().0
}

fn bound(keymap: &Keymap, spec: &str) -> Option<String> {
    keymap
        .command(&Chord::parse(spec).expect("a chord"))
        .map(|command| command.to_string())
}

#[test]
fn touching_nothing_writes_nothing() {
    assert!(
        overrides(&factory_rows()).is_empty(),
        "an untouched settings screen must not rewrite the keymap"
    );
}

#[test]
fn moving_one_command_leaves_every_other_default_alone() {
    let mut rows = factory_rows();
    let zen = rows.iter_mut().find(|row| row.command == "zen").unwrap();
    zen.keys = "Ctrl+E".to_string();
    zen.changed = true;

    let keymap = keymap_from(&rows);

    assert_eq!(bound(&keymap, "ctrl+e").as_deref(), Some("zen"));
    assert_eq!(bound(&keymap, "alt+z"), None, "the old chord is freed");

    assert_eq!(bound(&keymap, "ctrl+k").as_deref(), Some("palette"));
    assert_eq!(bound(&keymap, "ctrl+,").as_deref(), Some("settings"));
    assert_eq!(bound(&keymap, "ctrl+s").as_deref(), Some("save"));
    assert_eq!(bound(&keymap, "ctrl+z").as_deref(), Some("undo"));
    assert_eq!(bound(&keymap, "ctrl+c").as_deref(), Some("copy"));
}

#[test]
fn a_command_with_two_default_chords_gives_up_both_when_it_moves() {
    let mut rows = factory_rows();
    let redo = rows.iter_mut().find(|row| row.command == "redo").unwrap();
    redo.keys = "Ctrl+R".to_string();
    redo.changed = true;

    let keymap = keymap_from(&rows);

    assert_eq!(bound(&keymap, "ctrl+r").as_deref(), Some("redo"));
    assert_eq!(bound(&keymap, "ctrl+y"), None);
    assert_eq!(bound(&keymap, "ctrl+shift+z"), None);
    assert_eq!(bound(&keymap, "ctrl+z").as_deref(), Some("undo"));
}

#[test]
fn the_written_file_stays_small_and_readable() {
    let mut rows = factory_rows();
    let theme = rows.iter_mut().find(|row| row.command == "theme").unwrap();
    theme.keys = "Ctrl+Shift+T".to_string();
    theme.changed = true;

    let written = overrides(&rows);

    assert_eq!(
        written.len(),
        2,
        "one unbind and one binding, nothing else: {written:?}"
    );
}

#[test]
fn clearing_a_binding_frees_the_key_without_touching_the_rest() {
    let mut rows = factory_rows();
    let sidebar = rows.iter_mut().find(|row| row.command == "sidebar").unwrap();
    sidebar.keys.clear();
    sidebar.changed = true;

    let keymap = keymap_from(&rows);

    assert_eq!(bound(&keymap, "ctrl+b"), None);
    assert_eq!(bound(&keymap, "ctrl+k").as_deref(), Some("palette"));
    assert_eq!(bound(&keymap, "ctrl+,").as_deref(), Some("settings"));
}

#[test]
fn the_way_back_into_the_settings_screen_survives_editing_other_keys() {
    let mut rows = factory_rows();
    for row in rows.iter_mut() {
        if row.command != "settings" && row.command != "palette" {
            row.keys = format!("Ctrl+Alt+{}", row.command.chars().next().unwrap());
            row.changed = true;
        }
    }

    let keymap = keymap_from(&rows);

    assert_eq!(
        bound(&keymap, "ctrl+,").as_deref(),
        Some("settings"),
        "rebinding everything else must never lock the user out of settings"
    );
    assert_eq!(bound(&keymap, "ctrl+k").as_deref(), Some("palette"));
}

mod nonsense_bindings {
    use super::*;

    #[test]
    fn a_cyrillic_letter_is_not_a_chord() {
        assert_eq!(
            Chord::parse("Alt+\u{0419}"),
            None,
            "a layout-dependent letter can never match a physical key"
        );
    }

    #[test]
    fn a_file_carrying_one_falls_back_to_the_default() {
        let settings = Settings {
            keys: vec![Binding::new("Alt+\u{0419}", "zen")],
            ..Settings::default()
        };
        let (keymap, rejected) = settings.keymap();

        assert_eq!(rejected, vec!["Alt+\u{0419}".to_string()]);
        assert_eq!(
            bound(&keymap, "alt+z").as_deref(),
            Some("zen"),
            "the default must survive a binding the editor cannot read"
        );
    }
}
