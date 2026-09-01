use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Rust,
    TypeScript,
    Tsx,
    JavaScript,
    Json,
}

impl Language {
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

    pub const ALL: &'static [Language] = &[
        Language::Rust,
        Language::TypeScript,
        Language::Tsx,
        Language::JavaScript,
        Language::Json,
    ];
}
