use std::io::Write;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::{CoreError, Result};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirEntry {
    /// Relative to the workspace root.
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
}

/// Read a file as text. Rejects anything over `limit` or not valid UTF-8, so a
/// stray `read_file` on a 2 GB binary cannot take the editor down.
pub async fn read_text(path: &Path, limit: u64) -> Result<String> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let meta = std::fs::metadata(&path).map_err(|e| CoreError::io(&path, e))?;
        if meta.len() > limit {
            return Err(CoreError::TooLarge {
                path: path.clone(),
                size: meta.len(),
                limit,
            });
        }
        let bytes = std::fs::read(&path).map_err(|e| CoreError::io(&path, e))?;
        String::from_utf8(bytes).map_err(|_| CoreError::NotUtf8(path))
    })
    .await?
}

/// Write via a temp file in the same directory, then rename.
///
/// A crash mid-write leaves the previous file intact rather than a truncated
/// one — the file an agent is editing is never left half-written.
pub async fn write_atomic(path: &Path, contents: String) -> Result<()> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let dir = path
            .parent()
            .ok_or_else(|| CoreError::EscapesWorkspace(path.clone()))?;
        std::fs::create_dir_all(dir).map_err(|e| CoreError::io(dir, e))?;

        let mut tmp = tempfile::NamedTempFile::new_in(dir).map_err(|e| CoreError::io(dir, e))?;
        tmp.write_all(contents.as_bytes())
            .map_err(|e| CoreError::io(&path, e))?;
        tmp.as_file()
            .sync_all()
            .map_err(|e| CoreError::io(&path, e))?;
        tmp.persist(&path)
            .map_err(|e| CoreError::io(&path, e.error))?;
        Ok(())
    })
    .await?
}

pub async fn list_dir(dir: &Path, root: &Path) -> Result<Vec<DirEntry>> {
    let dir = dir.to_path_buf();
    let root = root.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let reader = std::fs::read_dir(&dir).map_err(|e| CoreError::io(&dir, e))?;
        let mut entries = Vec::new();
        for entry in reader {
            let entry = entry.map_err(|e| CoreError::io(&dir, e))?;
            let meta = match entry.metadata() {
                Ok(meta) => meta,
                // Vanished between listing and stat; nothing to report.
                Err(_) => continue,
            };
            let full = entry.path();
            entries.push(DirEntry {
                path: full.strip_prefix(&root).unwrap_or(&full).to_path_buf(),
                name: entry.file_name().to_string_lossy().into_owned(),
                is_dir: meta.is_dir(),
                size: meta.len(),
            });
        }
        // Directories first, then case-insensitive by name.
        entries.sort_by(|a, b| {
            b.is_dir
                .cmp(&a.is_dir)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
        Ok(entries)
    })
    .await?
}

pub async fn create_dir_all(path: &Path) -> Result<()> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        std::fs::create_dir_all(&path).map_err(|e| CoreError::io(&path, e))
    })
    .await?
}

pub async fn remove(path: &Path) -> Result<()> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let meta = std::fs::symlink_metadata(&path).map_err(|e| CoreError::io(&path, e))?;
        if meta.is_dir() {
            std::fs::remove_dir_all(&path).map_err(|e| CoreError::io(&path, e))
        } else {
            std::fs::remove_file(&path).map_err(|e| CoreError::io(&path, e))
        }
    })
    .await?
}

pub async fn rename(from: &Path, to: &Path) -> Result<()> {
    let from = from.to_path_buf();
    let to = to.to_path_buf();
    tokio::task::spawn_blocking(move || {
        if let Some(dir) = to.parent() {
            std::fs::create_dir_all(dir).map_err(|e| CoreError::io(dir, e))?;
        }
        std::fs::rename(&from, &to).map_err(|e| CoreError::io(&from, e))
    })
    .await?
}
