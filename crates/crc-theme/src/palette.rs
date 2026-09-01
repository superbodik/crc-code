//! Raw colour values, lifted from the design file.
//!
//! Nothing outside this module should name a colour by its shade. Screens ask
//! for a role — [`crate::chrome::Chrome::border`], not `NEUTRAL_300` — so a
//! second theme is a matter of swapping the role table, not hunting hex codes
//! through the renderer.

use crate::color::Rgba;

/// Warm greys. The whole shell sits on these; nothing is a pure neutral, which
/// is what keeps the light theme from looking clinical.
pub const NEUTRAL_0: Rgba = Rgba::hex(0xffffff);
pub const NEUTRAL_25: Rgba = Rgba::hex(0xfaf9f7);
pub const NEUTRAL_50: Rgba = Rgba::hex(0xf7f6f4);
pub const NEUTRAL_75: Rgba = Rgba::hex(0xf5f4f2);
pub const NEUTRAL_100: Rgba = Rgba::hex(0xf2f1ee);
pub const NEUTRAL_150: Rgba = Rgba::hex(0xefeeeb);
pub const NEUTRAL_200: Rgba = Rgba::hex(0xeeedea);
pub const NEUTRAL_250: Rgba = Rgba::hex(0xe7e6e3);
pub const NEUTRAL_300: Rgba = Rgba::hex(0xe0dfdb);
pub const NEUTRAL_350: Rgba = Rgba::hex(0xdcdbd7);
pub const NEUTRAL_400: Rgba = Rgba::hex(0xd5d4d0);
pub const NEUTRAL_450: Rgba = Rgba::hex(0xd3d2ce);
pub const NEUTRAL_500: Rgba = Rgba::hex(0xc2c1bb);
pub const NEUTRAL_550: Rgba = Rgba::hex(0xa9a8a2);
pub const NEUTRAL_600: Rgba = Rgba::hex(0x8a8983);
pub const NEUTRAL_650: Rgba = Rgba::hex(0x7a7973);
pub const NEUTRAL_700: Rgba = Rgba::hex(0x6e6d68);
pub const NEUTRAL_800: Rgba = Rgba::hex(0x57565f);
pub const NEUTRAL_850: Rgba = Rgba::hex(0x3a3936);
pub const NEUTRAL_900: Rgba = Rgba::hex(0x1b1b19);

/// Violet. Carries focus, the caret, the active tab and primary actions.
pub const ACCENT_50: Rgba = Rgba::hex(0xf1eefe);
pub const ACCENT_100: Rgba = Rgba::hex(0xe4e2fb);
pub const ACCENT_200: Rgba = Rgba::hex(0xc9c7f0);
pub const ACCENT_400: Rgba = Rgba::hex(0x8a7fd6);
pub const ACCENT_500: Rgba = Rgba::hex(0x7c5cff);
pub const ACCENT_600: Rgba = Rgba::hex(0x5b4bc4);
pub const ACCENT_700: Rgba = Rgba::hex(0x463a9e);
pub const ACCENT_800: Rgba = Rgba::hex(0x2f2a6b);

/// Green — additions and success.
pub const GREEN_50: Rgba = Rgba::hex(0xeff6f1);
pub const GREEN_200: Rgba = Rgba::hex(0x9dbcaa);
pub const GREEN_500: Rgba = Rgba::hex(0x4a7d5f);
pub const GREEN_600: Rgba = Rgba::hex(0x3d6b50);

/// Red — deletions and errors.
pub const RED_50: Rgba = Rgba::hex(0xfbf1ef);
pub const RED_200: Rgba = Rgba::hex(0xc9a49e);
pub const RED_500: Rgba = Rgba::hex(0xa4574d);
pub const RED_600: Rgba = Rgba::hex(0x8c6560);

/// Amber — modified files and warnings.
pub const AMBER_500: Rgba = Rgba::hex(0xb0873f);
/// Darkened for code, where amber sits at 13px on the buffer.
pub const AMBER_600: Rgba = Rgba::hex(0x896931);

/// Blue — information and links.
pub const BLUE_500: Rgba = Rgba::hex(0x2f6ba8);

/// A second violet, reserved for syntax keywords so they stay distinct from
/// the interactive accent.
pub const VIOLET_500: Rgba = Rgba::hex(0x8a5bd0);
/// Darkened for code. See the note on readability in `syntax.rs`.
pub const VIOLET_600: Rgba = Rgba::hex(0x8457c8);
