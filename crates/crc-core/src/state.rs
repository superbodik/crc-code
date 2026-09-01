use std::path::{Path, PathBuf};

use dashmap::DashMap;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub path: PathBuf,
    pub text: String,
    pub version: u64,
    pub dirty: bool,
}

#[derive(Debug, Default)]
pub struct Documents {
    docs: DashMap<PathBuf, Document>,
}

impl Documents {
    pub fn open(&self, path: PathBuf, text: String) -> Document {
        let doc = self
            .docs
            .entry(path.clone())
            .and_modify(|existing| {
                if existing.text != text {
                    existing.text = text.clone();
                    existing.version += 1;
                }
                existing.dirty = false;
            })
            .or_insert(Document {
                path,
                text,
                version: 1,
                dirty: false,
            });
        doc.clone()
    }

    pub fn get(&self, path: &Path) -> Option<Document> {
        self.docs.get(path).map(|d| d.clone())
    }

    pub fn version(&self, path: &Path) -> Option<u64> {
        self.docs.get(path).map(|d| d.version)
    }

    pub fn edit(&self, path: &Path, text: String) -> Option<u64> {
        self.docs.get_mut(path).map(|mut doc| {
            doc.text = text;
            doc.version += 1;
            doc.dirty = true;
            doc.version
        })
    }

    pub fn persisted(&self, path: PathBuf, text: String) -> u64 {
        let mut doc = self.docs.entry(path.clone()).or_insert(Document {
            path,
            text: String::new(),
            version: 0,
            dirty: false,
        });
        doc.text = text;
        doc.version += 1;
        doc.dirty = false;
        doc.version
    }

    pub fn close(&self, path: &Path) -> bool {
        self.docs.remove(path).is_some()
    }

    pub fn paths(&self) -> Vec<PathBuf> {
        self.docs.iter().map(|d| d.key().clone()).collect()
    }

    pub fn len(&self) -> usize {
        self.docs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.docs.is_empty()
    }
}
