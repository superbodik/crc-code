use std::path::PathBuf;

use serde::Serialize;
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Event {
    WorkspaceOpened { root: PathBuf },
    DocumentOpened { path: PathBuf, version: u64 },
    DocumentSaved { path: PathBuf, version: u64 },
    DocumentClosed { path: PathBuf },
    FileChanged { path: PathBuf, change: Change },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Change {
    Created,
    Modified,
    Removed,
}

#[derive(Debug, Clone)]
pub struct EventBus {
    tx: broadcast::Sender<Event>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        Self {
            tx: broadcast::channel(capacity).0,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.tx.subscribe()
    }

    pub fn emit(&self, event: Event) {
        let _ = self.tx.send(event);
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(1024)
    }
}
