//! Design tokens for the CRC Code editor.
//!
//! Taken from the design file at the root of the repository. Colours are named
//! by role rather than by shade — a panel asks for `chrome.border`, never for a
//! hex value — so the renderer, the syntax layer and the diff view stay in
//! agreement, and a second theme is a new role table rather than a search
//! through the drawing code.

pub mod chrome;
pub mod color;
pub mod density;
pub mod diff;
pub mod palette;
pub mod syntax;
pub mod theme;
pub mod typography;

pub use chrome::Chrome;
pub use color::Rgba;
pub use density::{Affordances, Density, Metrics};
pub use diff::DiffTheme;
pub use syntax::{Highlight, SyntaxTheme};
pub use theme::Theme;
pub use typography::{TypeScale, Weight};
