pub mod error;
pub mod geometry;
pub mod gpu;
pub mod layout;
pub mod view;
pub mod window;

pub use error::{Result, UiError};
pub use geometry::Rect;
pub use gpu::{
    FontKind, Frame, Gpu, Offscreen, Quad, QuadRenderer, Span, TextAlign, TextLayer, TextRun,
};
pub use layout::{Shell, ShellState};
pub use view::{CodeMetrics, EditorView, FileEntry, Tab};
pub use window::WindowRenderer;
