use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    C,
    Cpp,
    CSharp,
    Css,
    Html,
    Java,
    JavaScript,
    Json,
    Python,
    Rust,
    Tsx,
    TypeScript,
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
            "java" => Language::Java,
            "cpp" | "cc" | "cxx" | "c++" | "hpp" | "hh" | "hxx" => Language::Cpp,
            "c" | "h" => Language::C,
            "cs" => Language::CSharp,
            "py" | "pyi" | "pyw" => Language::Python,
            "html" | "htm" | "xhtml" => Language::Html,
            "css" => Language::Css,
            _ => return None,
        })
    }

    pub const fn name(self) -> &'static str {
        match self {
            Language::C => "C",
            Language::Cpp => "C++",
            Language::CSharp => "C#",
            Language::Css => "CSS",
            Language::Html => "HTML",
            Language::Java => "Java",
            Language::JavaScript => "JavaScript",
            Language::Json => "JSON",
            Language::Python => "Python",
            Language::Rust => "Rust",
            Language::Tsx => "TSX",
            Language::TypeScript => "TypeScript",
        }
    }

    pub(crate) fn grammar(self) -> tree_sitter::Language {
        match self {
            Language::C => tree_sitter_c::LANGUAGE,
            Language::Cpp => tree_sitter_cpp::LANGUAGE,
            Language::CSharp => tree_sitter_c_sharp::LANGUAGE,
            Language::Css => tree_sitter_css::LANGUAGE,
            Language::Html => tree_sitter_html::LANGUAGE,
            Language::Java => tree_sitter_java::LANGUAGE,
            Language::JavaScript => tree_sitter_javascript::LANGUAGE,
            Language::Json => tree_sitter_json::LANGUAGE,
            Language::Python => tree_sitter_python::LANGUAGE,
            Language::Rust => tree_sitter_rust::LANGUAGE,
            Language::Tsx => tree_sitter_typescript::LANGUAGE_TSX,
            Language::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT,
        }
        .into()
    }

    pub(crate) fn highlights_query(self) -> String {
        match self {
            Language::C => tree_sitter_c::HIGHLIGHT_QUERY.to_string(),
            Language::Cpp => format!(
                "{}\n{}",
                tree_sitter_c::HIGHLIGHT_QUERY,
                tree_sitter_cpp::HIGHLIGHT_QUERY
            ),
            Language::CSharp => tree_sitter_c_sharp::HIGHLIGHTS_QUERY.to_string(),
            Language::Css => tree_sitter_css::HIGHLIGHTS_QUERY.to_string(),
            Language::Html => tree_sitter_html::HIGHLIGHTS_QUERY.to_string(),
            Language::Java => tree_sitter_java::HIGHLIGHTS_QUERY.to_string(),
            Language::JavaScript => format!(
                "{}\n{}",
                tree_sitter_javascript::HIGHLIGHT_QUERY,
                tree_sitter_javascript::JSX_HIGHLIGHT_QUERY
            ),
            Language::Json => tree_sitter_json::HIGHLIGHTS_QUERY.to_string(),
            Language::Python => tree_sitter_python::HIGHLIGHTS_QUERY.to_string(),
            Language::Rust => tree_sitter_rust::HIGHLIGHTS_QUERY.to_string(),
            Language::Tsx => format!(
                "{}\n{}\n{}",
                tree_sitter_javascript::HIGHLIGHT_QUERY,
                tree_sitter_javascript::JSX_HIGHLIGHT_QUERY,
                tree_sitter_typescript::HIGHLIGHTS_QUERY
            ),
            Language::TypeScript => format!(
                "{}\n{}",
                tree_sitter_javascript::HIGHLIGHT_QUERY,
                tree_sitter_typescript::HIGHLIGHTS_QUERY
            ),
        }
    }

    pub const ALL: &'static [Language] = &[
        Language::C,
        Language::Cpp,
        Language::CSharp,
        Language::Css,
        Language::Html,
        Language::Java,
        Language::JavaScript,
        Language::Json,
        Language::Python,
        Language::Rust,
        Language::Tsx,
        Language::TypeScript,
    ];
}
