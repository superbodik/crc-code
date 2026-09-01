use std::path::Path;

/// A language the editor can parse.
///
/// Adding one is a row in each `match` below plus a grammar dependency —
/// deliberately boring, because the list is going to get long.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Rust,
    TypeScript,
    /// TypeScript with JSX. A separate grammar, not a flag.
    Tsx,
    JavaScript,
    Json,
}

impl Language {
    /// Guess from a file name. `None` means no grammar, which is not an error —
    /// the buffer just renders as plain text.
    pub fn from_path(path: impl AsRef<Path>) -> Option<Self> {
        let extension = path.as_ref().extension()?.to_str()?;
        Self::from_extension(extension)
    }

    pub fn from_extension(extension: &str) -> Option<Self> {
        Some(match extension.to_ascii_lowercase().as_str() {
            "rs" => Language::Rust,
            "ts" | "mts" | "cts" => Language::TypeScript,
            "tsx" => Language::Tsx,
            "js" | "mjs" | "cjs" | "jsx" => Language::JavaScript,
            "json" | "jsonc" => Language::Json,
            _ => return None,
        })
    }

    /// As shown in the status bar.
    pub const fn name(self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::TypeScript => "TypeScript",
            Language::Tsx => "TSX",
            Language::JavaScript => "JavaScript",
            Language::Json => "JSON",
        }
    }

    pub(crate) fn grammar(self) -> tree_sitter::Language {
        match self {
            Language::Rust => tree_sitter_rust::LANGUAGE,
            Language::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT,
            Language::Tsx => tree_sitter_typescript::LANGUAGE_TSX,
            Language::JavaScript => tree_sitter_javascript::LANGUAGE,
            Language::Json => tree_sitter_json::LANGUAGE,
        }
        .into()
    }

    /// The highlight query.
    ///
    /// The TypeScript queries only carry what TypeScript adds, so the
    /// JavaScript query has to come first or half the file goes uncoloured.
    pub(crate) fn highlights_query(self) -> String {
        match self {
            Language::Rust => tree_sitter_rust::HIGHLIGHTS_QUERY.to_string(),
            Language::TypeScript => format!(
                "{}\n{}",
                tree_sitter_javascript::HIGHLIGHT_QUERY,
                tree_sitter_typescript::HIGHLIGHTS_QUERY
            ),
            Language::Tsx => format!(
                "{}\n{}\n{}",
                tree_sitter_javascript::HIGHLIGHT_QUERY,
                tree_sitter_javascript::JSX_HIGHLIGHT_QUERY,
                tree_sitter_typescript::HIGHLIGHTS_QUERY
            ),
            Language::JavaScript => format!(
                "{}\n{}",
                tree_sitter_javascript::HIGHLIGHT_QUERY,
                tree_sitter_javascript::JSX_HIGHLIGHT_QUERY
            ),
            Language::Json => tree_sitter_json::HIGHLIGHTS_QUERY.to_string(),
        }
    }

    /// Every language the editor knows, for settings and tests.
    pub const ALL: &'static [Language] = &[
        Language::Rust,
        Language::TypeScript,
        Language::Tsx,
        Language::JavaScript,
        Language::Json,
    ];
}
