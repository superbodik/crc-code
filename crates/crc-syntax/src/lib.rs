pub mod error;
pub mod highlight;
pub mod language;
pub mod parser;

pub use error::{Result, SyntaxError};
pub use highlight::{HighlightSpan, resolve, role_for};
pub use language::Language;
pub use parser::SyntaxTree;
