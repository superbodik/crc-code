use crc_config::{Binding, Chord, Keymap, keymap};
use crc_ui::view::settings::BindingRow;

pub fn overrides(rows: &[BindingRow]) -> Vec<Binding> {
    let (factory, _) = Keymap::from_bindings(&keymap::defaults());
    let mut keys = Vec::new();

    for row in rows.iter().filter(|row| row.changed) {
        let wanted = Chord::parse(&row.keys);

        let owned: Vec<Chord> = factory
            .iter()
            .filter(|(_, command)| *command == row.command)
            .map(|(chord, _)| *chord)
            .collect();

        for chord in owned {
            if Some(chord) != wanted {
                keys.push(Binding::new(chord.label(), ""));
            }
        }

        if let Some(chord) = wanted {
            keys.push(Binding::new(chord.label(), row.command.clone()));
        }
    }

    keys
}
