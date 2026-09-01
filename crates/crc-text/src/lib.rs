//! Text buffer for the CRC Code editor.
//!
//! Storage is a rope, so edit cost does not grow with file size. Everything
//! that mutates a buffer is an [`Edit`] — insert, delete and replace are the
//! same shape — which gives undo history and change notification a single case
//! to handle.
//!
//! Offsets are character offsets, never bytes, so a position can never land
//! inside a multi-byte character. [`Point`] converts to and from line/column.

pub mod author;
pub mod buffer;
pub mod edit;
pub mod history;
pub mod point;
pub mod selection;

pub use author::AuthorId;
pub use buffer::Buffer;
pub use edit::{Change, Edit, rebase};
pub use history::{History, Transaction};
pub use point::Point;
pub use selection::Selection;
