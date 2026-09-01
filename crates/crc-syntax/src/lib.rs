//! Incremental parsing and highlighting.
//!
//! Wraps tree-sitter and maps its capture vocabulary onto the small set of
//! roles the theme defines, so the renderer never sees a grammar's naming and
//! a new language does not mean a new colour.
//!
//! Offsets here are byte offsets, which is what tree-sitter speaks. The buffer
//! converts at the boundary.

pub mod error;
pub mod highlight;
pub mod language;
pub mod parser;

pub use error::{Result, SyntaxError};
pub use highlight::{HighlightSpan, resolve, role_for};
pub use language::Language;
pub use parser::SyntaxTree;
