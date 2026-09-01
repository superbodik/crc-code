use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::broadcast;

use crate::error::{CoreError, Result};
use crate::event::{Event, EventBus};
use crate::fs::{self, DirEntry, FileMatches, TextQuery};
use crate::state::{Document, Documents};
use crate::workspace::Workspace;

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

pub struct Engine {
    workspace: Workspace,
    documents: Documents,
    bus: EventBus,
    limits: Limits,
    _watcher: fs::FileWatcher,
}

impl Engine {
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

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.bus.subscribe()
    }

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

    pub async fn search_text(&self, query: TextQuery) -> Result<Vec<FileMatches>> {
        let root = self.workspace.root().to_path_buf();
        tokio::task::spawn_blocking(move || fs::search::search_text(&root, &query)).await?
    }

    pub async fn find_files(&self, query: impl Into<String>, limit: usize) -> Result<Vec<PathBuf>> {
        let root = self.workspace.root().to_path_buf();
        let query = query.into();
        Ok(
            tokio::task::spawn_blocking(move || fs::search::find_files(&root, &query, limit))
                .await?,
        )
    }
}
