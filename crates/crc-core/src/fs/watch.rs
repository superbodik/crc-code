use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use notify::{EventKind, RecursiveMode, Watcher};
use tokio::sync::mpsc;

use crate::error::Result;
use crate::event::{Change, Event, EventBus};

/// Watches the workspace and republishes changes on the [`EventBus`].
///
/// Dropping it stops the watch.
pub struct FileWatcher {
    _watcher: notify::RecommendedWatcher,
    task: tokio::task::JoinHandle<()>,
}

impl Drop for FileWatcher {
    fn drop(&mut self) {
        self.task.abort();
    }
}

/// Start watching `root`.
///
/// Changes are coalesced: a burst on one path settles into one event per path
/// once `debounce` passes quietly, so a `cargo build` cannot flood the UI with
/// thousands of events.
pub fn spawn(root: &Path, bus: EventBus, debounce: Duration) -> Result<FileWatcher> {
    let (tx, mut rx) = mpsc::unbounded_channel::<(PathBuf, Change)>();
    let root_owned = root.to_path_buf();

    let mut watcher = notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
        let Ok(event) = result else { return };
        let change = match event.kind {
            EventKind::Create(_) => Change::Created,
            EventKind::Remove(_) => Change::Removed,
            EventKind::Modify(_) => Change::Modified,
            _ => return,
        };
        for path in event.paths {
            if is_noise(&path) {
                continue;
            }
            let relative = path
                .strip_prefix(&root_owned)
                .unwrap_or(&path)
                .to_path_buf();
            let _ = tx.send((relative, change));
        }
    })?;
    watcher.watch(root, RecursiveMode::Recursive)?;

    let task = tokio::spawn(async move {
        let mut pending: HashMap<PathBuf, Change> = HashMap::new();
        loop {
            let quiet = tokio::time::sleep(debounce);
            tokio::pin!(quiet);

            tokio::select! {
                incoming = rx.recv() => match incoming {
                    Some((path, change)) => {
                        pending.entry(path).or_insert(change);
                    }
                    None => break,
                },
                _ = &mut quiet, if !pending.is_empty() => {
                    for (path, change) in pending.drain() {
                        bus.emit(Event::FileChanged { path, change });
                    }
                }
            }
        }
    });

    Ok(FileWatcher {
        _watcher: watcher,
        task,
    })
}

fn is_noise(path: &Path) -> bool {
    path.components().any(|c| {
        matches!(
            c.as_os_str().to_str(),
            Some(".git" | "target" | "node_modules" | "dist" | ".next")
        )
    })
}
