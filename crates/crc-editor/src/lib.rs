pub mod document;
pub mod documents;
pub mod motion;

pub use document::Document;
pub use documents::Documents;
pub use motion::{CharClass, Motion, class_of};

pub const AUTOSAVE_IDLE_MS: u64 = 800;
