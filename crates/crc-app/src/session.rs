use std::path::{Path, PathBuf};
use std::sync::Arc;

use crc_core::{Engine, Limits};
use crc_editor::Document;
use crc_ui::view::{EditorView, FileEntry, Tab};

pub struct Session {
    runtime: tokio::runtime::Runtime,
    engine: Arc<Engine>,
    document: Option<Document>,
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
            document: None,
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
        self.document.as_mut()
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
        if self.document.as_ref().is_some_and(|doc| doc.path() == path) {
            return false;
        }
        self.save().ok();
        self.open_file(&path).is_ok()
    }

    pub fn open_file(&mut self, path: &Path) -> anyhow::Result<()> {
        let document = self.runtime.block_on(self.engine.read_file(path))?;
        let document = Document::open(path, document.text);

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();

        self.view.tabs = vec![Tab::new(name.clone()).active()];
        for entry in &mut self.view.files {
            entry.selected = entry.name == name;
        }

        self.view.scroll_line = 0;
        self.document = Some(document);
        self.sync();
        Ok(())
    }

    pub fn save(&mut self) -> anyhow::Result<bool> {
        let Some(document) = self.document.as_mut() else {
            return Ok(false);
        };
        if !document.is_dirty() {
            return Ok(false);
        }

        let path = document.path().to_path_buf();
        let text = document.text().to_string();
        self.runtime
            .block_on(self.engine.write_file(&path, text, None))?;

        document.mark_saved();
        self.sync();
        Ok(true)
    }

    pub fn sync(&mut self) {
        let Some(document) = self.document.as_ref() else {
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
