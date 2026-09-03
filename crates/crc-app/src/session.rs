use std::path::{Path, PathBuf};
use std::sync::Arc;

use crc_core::{Engine, Limits};
use crc_editor::{Document, Documents};
use crc_ui::view::{EditorView, FileEntry, Tab};

pub struct Session {
    runtime: tokio::runtime::Runtime,
    engine: Arc<Engine>,
    documents: Documents,
    files: Vec<PathBuf>,
    pub view: EditorView,
}

impl Session {
    pub fn open(root: &Path) -> anyhow::Result<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        let engine = runtime.block_on(async { Engine::open(root, Limits::default()) })?;

        let project = engine
            .root()
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "workspace".to_string());

        let mut session = Self {
            runtime,
            engine,
            documents: Documents::new(),
            files: Vec::new(),
            view: EditorView {
                project,
                branch: "main".to_string(),
                focused: true,
                ..EditorView::default()
            },
        };

        session.load_tree();
        if let Some(first) = session.pick_file() {
            session.open_file(&first)?;
        }
        Ok(session)
    }

    pub fn root(&self) -> &Path {
        self.engine.root()
    }

    pub fn document(&mut self) -> Option<&mut Document> {
        self.documents.active_mut()
    }

    fn load_tree(&mut self) {
        self.files = self
            .runtime
            .block_on(self.engine.find_files("", 400))
            .unwrap_or_default()
            .into_iter()
            .take(200)
            .collect();

        self.view.files = self
            .files
            .iter()
            .map(|path| {
                let depth = path.components().count().saturating_sub(1);
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                FileEntry::file(name, depth)
            })
            .collect();
    }

    fn pick_file(&self) -> Option<PathBuf> {
        self.files
            .iter()
            .find(|path| path.extension().is_some_and(|ext| ext == "rs"))
            .or_else(|| self.files.first())
            .cloned()
    }

    pub fn open_row(&mut self, row: usize) -> bool {
        let Some(path) = self.files.get(row).cloned() else {
            return false;
        };
        self.open_file(&path).is_ok()
    }

    pub fn open_file(&mut self, path: &Path) -> anyhow::Result<()> {
        if let Some(index) = self.documents.index_of(path) {
            self.documents.activate(index);
            self.view.scroll_line = 0;
            self.sync();
            return Ok(());
        }

        let opened = self.runtime.block_on(self.engine.read_file(path))?;
        self.documents.open(Document::open(path, opened.text));
        self.view.scroll_line = 0;
        self.sync();
        Ok(())
    }

    pub fn active_tab(&self) -> Option<usize> {
        self.documents.active_index()
    }

    pub fn activate_tab(&mut self, index: usize) -> bool {
        if !self.documents.activate(index) {
            return false;
        }
        self.view.scroll_line = 0;
        self.sync();
        true
    }

    pub fn close_tab(&mut self, index: usize) -> bool {
        if self.documents.get(index).is_none() {
            return false;
        }
        self.save_index(index);
        self.documents.close(index);
        self.view.scroll_line = 0;
        self.sync();
        true
    }

    fn save_index(&mut self, index: usize) -> bool {
        let Some(document) = self.documents.get(index) else {
            return false;
        };
        if !document.is_dirty() {
            return false;
        }

        let path = document.path().to_path_buf();
        let text = document.text().to_string();
        match self
            .runtime
            .block_on(self.engine.write_file(&path, text, None))
        {
            Ok(_) => {
                if let Some(document) = self.documents.get_mut(index) {
                    document.mark_saved();
                }
                true
            }
            Err(error) => {
                tracing::error!("could not save {}: {error}", path.display());
                false
            }
        }
    }

    pub fn save(&mut self) -> anyhow::Result<bool> {
        let Some(index) = self.documents.active_index() else {
            return Ok(false);
        };
        let saved = self.save_index(index);
        if saved {
            self.sync();
        }
        Ok(saved)
    }

    pub fn save_all(&mut self) {
        for index in 0..self.documents.len() {
            self.save_index(index);
        }
        self.sync();
    }

    pub fn sync(&mut self) {
        let active = self.documents.active_index();

        self.view.tabs = self
            .documents
            .iter()
            .enumerate()
            .map(|(index, document)| {
                let name = document
                    .path()
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let mut tab = Tab::new(name);
                tab.active = active == Some(index);
                tab.modified = document.is_dirty();
                tab
            })
            .collect();

        let open_names: Vec<String> = self.view.tabs.iter().map(|tab| tab.name.clone()).collect();
        let selected = self
            .documents
            .active()
            .and_then(|document| document.path().file_name())
            .map(|name| name.to_string_lossy().into_owned());
        for entry in &mut self.view.files {
            entry.selected = selected.as_deref() == Some(entry.name.as_str());
            entry.modified = open_names.contains(&entry.name)
                && self
                    .documents
                    .iter()
                    .any(|d| d.is_dirty() && d.path().ends_with(&entry.name));
        }

        let Some(document) = self.documents.active() else {
            self.view.text = String::new();
            self.view.highlights = Vec::new();
            self.view.selection = None;
            self.view.dirty = false;
            self.view.language = "—".to_string();
            self.view.cursor_line = 0;
            self.view.cursor_column = 0;
            return;
        };

        let cursor = document.cursor();
        self.view.text = document.text().to_string();
        self.view.highlights = document.highlights();
        self.view.selection = document.selected_bytes();
        self.view.cursor_line = cursor.line;
        self.view.cursor_column = cursor.column;
        self.view.dirty = document.is_dirty();
        self.view.language = document
            .language()
            .map(|language| language.name())
            .unwrap_or("Текст")
            .to_string();
    }

    pub fn scroll_by(&mut self, lines: isize) {
        let last = self.view.line_count().saturating_sub(1);
        let target = self.view.scroll_line as isize + lines;
        self.view.scroll_line = target.clamp(0, last as isize) as usize;
    }

    pub fn follow_cursor(&mut self, rows: usize) {
        if rows == 0 {
            return;
        }
        let line = self.view.cursor_line;
        if line < self.view.scroll_line {
            self.view.scroll_line = line;
        } else if line >= self.view.scroll_line + rows {
            self.view.scroll_line = line + 1 - rows;
        }
    }
}
