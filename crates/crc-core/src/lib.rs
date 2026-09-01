//! Local engine for the CRC Code editor.
//!
//! Owns the open workspace, its buffers, and every operation that touches the
//! user's disk. The UI, the plugin host and the AI agents are all clients of
//! [`Engine`] - which is what makes its workspace sandbox meaningful: there is
//! no second path to the filesystem to secure.

pub mod engine;
pub mod error;
pub mod event;
pub mod fs;
pub mod state;
pub mod workspace;

pub use engine::{Engine, Limits};
pub use error::{CoreError, Result};
pub use event::{Change, Event, EventBus};
pub use state::{Document, Documents};
pub use workspace::Workspace;
