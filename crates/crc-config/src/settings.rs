use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::keymap::{Binding, Keymap, defaults};
use crate::recent::Recent;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Visible {
    pub rail: bool,
    pub explorer: bool,
    pub tabs: bool,
    pub breadcrumbs: bool,
    pub minimap: bool,
    pub panel: bool,
    pub status_bar: bool,
}

impl Default for Visible {
    fn default() -> Self {
        Self {
            rail: true,
            explorer: true,
            tabs: true,
            breadcrumbs: true,
            minimap: true,
            panel: true,
            status_bar: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub appearance: String,
    pub density: String,
    pub code_size: f32,
    pub autosave_ms: u64,
    pub visible: Visible,
    pub recent: Vec<Recent>,
    pub keys: Vec<Binding>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            appearance: "dark".to_string(),
            density: "balanced".to_string(),
            code_size: 13.0,
            autosave_ms: 800,
            visible: Visible::default(),
            recent: Vec::new(),
            keys: Vec::new(),
        }
    }
}

impl Settings {
    pub fn keymap(&self) -> (Keymap, Vec<String>) {
        let mut bindings = defaults();
        bindings.extend(self.keys.iter().cloned());
        Keymap::from_bindings(&bindings)
    }

    pub fn load(path: &Path) -> (Settings, Option<String>) {
        let Ok(text) = std::fs::read_to_string(path) else {
            return (Settings::default(), None);
        };
        match toml::from_str(&text) {
            Ok(settings) => (settings, None),
            Err(error) => (Settings::default(), Some(error.to_string())),
        }
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(self)
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        std::fs::write(path, text)
    }

    pub fn remember(&mut self, root: &Path, now: u64) {
        crate::recent::remember(&mut self.recent, root, now, crate::recent::LIMIT);
    }
}

pub fn config_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("CRC_CONFIG_DIR") {
        return PathBuf::from(dir);
    }
    if let Ok(appdata) = std::env::var("APPDATA") {
        return PathBuf::from(appdata).join("CRC Code");
    }
    if let Ok(home) = std::env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(home).join("crc-code");
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".config").join("crc-code");
    }
    PathBuf::from(".crc-code")
}

pub fn settings_file() -> PathBuf {
    config_dir().join("settings.toml")
}
