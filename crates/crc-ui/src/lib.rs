pub mod error;
pub mod geometry;
pub mod font;
pub mod icon;
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
pub use view::{Band, Mark, TabHit, VisibleText, bands, buffer_point, explorer_row, tree};
pub use view::{
    CodeMetrics, Edge, EditorView, FileEntry, Tab, WindowControl, control_at, control_rect,
    is_drag_handle, resize_edge,
};
pub use window::WindowRenderer;
