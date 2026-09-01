pub mod ops;
pub mod search;
pub mod watch;

pub use ops::DirEntry;
pub use search::{FileMatches, LineMatch, TextQuery};
pub use watch::FileWatcher;
