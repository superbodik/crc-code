use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::broadcast;

use crate::error::{CoreError, Result};
use crate::event::{Event, EventBus};
use crate::fs::{self, DirEntry, FileMatches, TextQuery};
use crate::state::{Document, Documents};
use crate::workspace::Workspace;

/// Ceilings that keep one bad request from taking the editor down.
#[derive(Debug, Clone)]
pub struct Limits {
    pub max_text_bytes: u64,
    pub watch_debounce: Duration,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_text_bytes: 8 * 1024 * 1024,
            watch_debounce: Duration::from_millis(120),
        }
    }
}

/// The local engine: one open workspace, its buffers, and the operations over
/// them.
///
/// This is the only door to the user's disk. The UI, plugins and agents are all
/// clients of it, so the workspace sandbox and the size limits hold for every
/// one of them - there is no second path to guard.
pub struct Engine {
    workspace: Workspace,
    documents: Documents,
    bus: EventBus,
    limits: Limits,
    _watcher: fs::FileWatcher,
}

impl Engine {
    /// Open a workspace and start watching it.
    pub fn open(root: impl AsRef<Path>, limits: Limits) -> Result<Arc<Self>> {
        let workspace = Workspace::open(root)?;
        let bus = EventBus::default();
        let watcher = fs::watch::spawn(workspace.root(), bus.clone(), limits.watch_debounce)?;

        bus.emit(Event::WorkspaceOpened {
            root: workspace.root().to_path_buf(),
        });

        Ok(Arc::new(Self {
            workspace,
            documents: Documents::default(),
            bus,
            limits,
            _watcher: watcher,
        }))
    }

    pub fn root(&self) -> &Path {
        self.workspace.root()
    }

    pub fn documents(&self) -> &Documents {
        &self.documents
    }

    /// Every engine event, from this moment on.
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.bus.subscribe()
    }

    /// Read a file and keep it as an open buffer.
    pub async fn read_file(&self, path: impl AsRef<Path>) -> Result<Document> {
        let absolute = self.workspace.resolve(path)?;
        let text = fs::ops::read_text(&absolute, self.limits.max_text_bytes).await?;
        let relative = self.workspace.relative(&absolute).to_path_buf();

        let doc = self.documents.open(relative.clone(), text);
        self.bus.emit(Event::DocumentOpened {
            path: relative,
            version: doc.version,
        });
        Ok(doc)
    }

    /// Write a file to disk.
    ///
    /// `expected_version` makes the write optimistic: pass the version the
    /// caller last saw and the write is refused if the buffer moved since. This
    /// is what stops an agent from pasting a diff over a file the user edited
    /// in the meantime.
    pub async fn write_file(
        &self,
        path: impl AsRef<Path>,
        contents: String,
        expected_version: Option<u64>,
    ) -> Result<u64> {
        let absolute = self.workspace.resolve(&path)?;
        let relative = self.workspace.relative(&absolute).to_path_buf();

        if let (Some(expected), Some(actual)) =
            (expected_version, self.documents.version(&relative))
            && expected != actual
        {
            return Err(CoreError::VersionConflict {
                path: relative,
                expected,
                actual,
            });
        }

        fs::ops::write_atomic(&absolute, contents.clone()).await?;
        let version = self.documents.persisted(relative.clone(), contents);
        self.bus.emit(Event::DocumentSaved {
            path: relative,
            version,
        });
        Ok(version)
    }

    pub fn close_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let absolute = self.workspace.resolve(path)?;
        let relative = self.workspace.relative(&absolute).to_path_buf();
        if self.documents.close(&relative) {
            self.bus.emit(Event::DocumentClosed { path: relative });
        }
        Ok(())
    }

    pub async fn list_dir(&self, path: impl AsRef<Path>) -> Result<Vec<DirEntry>> {
        let absolute = self.workspace.resolve(path)?;
        fs::ops::list_dir(&absolute, self.workspace.root()).await
    }

    pub async fn create_dir(&self, path: impl AsRef<Path>) -> Result<()> {
        let absolute = self.workspace.resolve(path)?;
        fs::ops::create_dir_all(&absolute).await
    }

    pub async fn remove(&self, path: impl AsRef<Path>) -> Result<()> {
        let absolute = self.workspace.resolve(path)?;
        let relative = self.workspace.relative(&absolute).to_path_buf();
        fs::ops::remove(&absolute).await?;
        self.documents.close(&relative);
        Ok(())
    }

    pub async fn rename(&self, from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
        let from_abs = self.workspace.resolve(from)?;
        let to_abs = self.workspace.resolve(to)?;
        let from_rel = self.workspace.relative(&from_abs).to_path_buf();
        fs::ops::rename(&from_abs, &to_abs).await?;
        self.documents.close(&from_rel);
        Ok(())
    }

    /// Content search across the workspace.
    pub async fn search_text(&self, query: TextQuery) -> Result<Vec<FileMatches>> {
        let root = self.workspace.root().to_path_buf();
        tokio::task::spawn_blocking(move || fs::search::search_text(&root, &query)).await?
    }

    /// Fuzzy filename lookup.
    pub async fn find_files(&self, query: impl Into<String>, limit: usize) -> Result<Vec<PathBuf>> {
        let root = self.workspace.root().to_path_buf();
        let query = query.into();
        Ok(
            tokio::task::spawn_blocking(move || fs::search::find_files(&root, &query, limit))
                .await?,
        )
    }
}
