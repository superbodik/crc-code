use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const LIMIT: usize = 12;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recent {
    pub path: PathBuf,
    pub name: String,
    pub opened_at: u64,
}

impl Recent {
    pub fn new(path: impl Into<PathBuf>, now: u64) -> Self {
        let path = path.into();
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());
        Self {
            path,
            name,
            opened_at: now,
        }
    }
}

pub fn remember(list: &mut Vec<Recent>, path: &Path, now: u64, limit: usize) {
    list.retain(|entry| entry.path != path);
    list.insert(0, Recent::new(path, now));
    list.truncate(limit);
}

pub fn forget(list: &mut Vec<Recent>, path: &Path) -> bool {
    let before = list.len();
    list.retain(|entry| entry.path != path);
    list.len() != before
}

pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0)
}

pub fn since(opened_at: u64, now: u64) -> String {
    let elapsed = now.saturating_sub(opened_at);
    match elapsed {
        0..=59 => "только что".to_string(),
        60..=3599 => format!("{} мин назад", elapsed / 60),
        3600..=86399 => format!("{} ч назад", elapsed / 3600),
        86400..=172799 => "вчера".to_string(),
        _ => format!("{} дн назад", elapsed / 86400),
    }
}
