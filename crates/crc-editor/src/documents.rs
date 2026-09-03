use std::path::Path;

use crate::document::Document;

#[derive(Default)]
pub struct Documents {
    open: Vec<Document>,
    active: usize,
}

impl Documents {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.open.len()
    }

    pub fn is_empty(&self) -> bool {
        self.open.is_empty()
    }

    pub fn active_index(&self) -> Option<usize> {
        (!self.open.is_empty()).then_some(self.active)
    }

    pub fn active(&self) -> Option<&Document> {
        self.open.get(self.active)
    }

    pub fn active_mut(&mut self) -> Option<&mut Document> {
        self.open.get_mut(self.active)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Document> {
        self.open.iter()
    }

    pub fn get(&self, index: usize) -> Option<&Document> {
        self.open.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Document> {
        self.open.get_mut(index)
    }

    pub fn index_of(&self, path: &Path) -> Option<usize> {
        self.open.iter().position(|doc| doc.path() == path)
    }

    pub fn dirty_paths(&self) -> Vec<&Path> {
        self.open
            .iter()
            .filter(|doc| doc.is_dirty())
            .map(|doc| doc.path())
            .collect()
    }

    pub fn open(&mut self, document: Document) -> usize {
        match self.index_of(document.path()) {
            Some(index) => {
                self.active = index;
                index
            }
            None => {
                self.open.push(document);
                self.active = self.open.len() - 1;
                self.active
            }
        }
    }

    pub fn activate(&mut self, index: usize) -> bool {
        if index >= self.open.len() || index == self.active {
            return false;
        }
        self.active = index;
        true
    }

    pub fn close(&mut self, index: usize) -> Option<Document> {
        if index >= self.open.len() {
            return None;
        }
        let closed = self.open.remove(index);

        self.active = if self.open.is_empty() {
            0
        } else if index < self.active {
            self.active - 1
        } else {
            self.active.min(self.open.len() - 1)
        };

        Some(closed)
    }

    pub fn close_active(&mut self) -> Option<Document> {
        self.active_index().and_then(|index| self.close(index))
    }
}
