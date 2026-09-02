use std::path::{Path, PathBuf};
use std::sync::Arc;

use crc_core::{Engine, Limits};
use crc_syntax::{Language, SyntaxTree};
use crc_ui::view::{EditorView, FileEntry, Tab};

pub struct Session {
    runtime: tokio::runtime::Runtime,
    engine: Arc<Engine>,
    tree: Option<SyntaxTree>,
    open: Option<PathBuf>,
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
            tree: None,
            open: None,
            view: EditorView {
                project,
                branch: "main".to_string(),
                focused: true,
                ..EditorView::default()
            },
        };

        session.load_tree()?;
        if let Some(first) = session.pick_file() {
            session.open_file(&first)?;
        }
        Ok(session)
    }

    pub fn root(&self) -> &Path {
        self.engine.root()
    }

    fn load_tree(&mut self) -> anyhow::Result<()> {
        let paths = self
            .runtime
            .block_on(self.engine.find_files("", 400))
            .unwrap_or_default();

        let mut entries = Vec::new();
        for path in paths.iter().take(60) {
            let depth = path.components().count().saturating_sub(1);
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            entries.push(FileEntry::file(name, depth));
        }
        self.view.files = entries;
        Ok(())
    }

    fn pick_file(&self) -> Option<PathBuf> {
        let paths = self
            .runtime
            .block_on(self.engine.find_files("", 400))
            .unwrap_or_default();

        paths
            .iter()
            .find(|path| path.extension().is_some_and(|ext| ext == "rs"))
            .or_else(|| paths.first())
            .cloned()
    }

    pub fn open_file(&mut self, path: &Path) -> anyhow::Result<()> {
        let document = self.runtime.block_on(self.engine.read_file(path))?;

        let language = Language::from_path(path);
        self.tree = match language {
            Some(language) => {
                let mut tree = SyntaxTree::new(language)?;
                tree.parse(&document.text)?;
                Some(tree)
            }
            None => None,
        };

        self.view.highlights = match (&self.tree, language) {
            (Some(tree), _) => tree
                .highlights(&document.text)
                .into_iter()
                .map(|span| (span.range, span.highlight))
                .collect(),
            _ => Vec::new(),
        };

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();

        self.view.language = language.map(|l| l.name()).unwrap_or("Текст").to_string();
        self.view.text = document.text;
        self.view.cursor_line = 0;
        self.view.cursor_column = 0;
        self.view.scroll_line = 0;
        self.view.tabs = vec![Tab::new(name.clone()).active()];

        for entry in &mut self.view.files {
            entry.selected = entry.name == name;
        }
        self.open = Some(path.to_path_buf());
        Ok(())
    }

    pub fn move_cursor(&mut self, delta: isize, rows: usize) {
        let last = self.view.line_count().saturating_sub(1);
        let line = self.view.cursor_line as isize + delta;
        self.view.cursor_line = line.clamp(0, last as isize) as usize;

        if self.view.cursor_line < self.view.scroll_line {
            self.view.scroll_line = self.view.cursor_line;
        } else if rows > 0 && self.view.cursor_line >= self.view.scroll_line + rows {
            self.view.scroll_line = self.view.cursor_line + 1 - rows;
        }
    }
}
