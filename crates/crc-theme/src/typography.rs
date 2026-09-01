/// Font stacks. The design specifies IBM Plex; the fallbacks keep the shell
/// legible before the fonts are loaded or if they are missing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontFamily {
    pub primary: &'static str,
    pub fallbacks: &'static [&'static str],
}

pub const SANS: FontFamily = FontFamily {
    primary: "IBM Plex Sans",
    fallbacks: &["Segoe UI", "Helvetica", "sans-serif"],
};

pub const MONO: FontFamily = FontFamily {
    primary: "IBM Plex Mono",
    fallbacks: &["Cascadia Mono", "Consolas", "monospace"],
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weight {
    Regular = 400,
    Medium = 500,
    Semibold = 600,
}

/// The type scale, in logical pixels before display scaling.
///
/// It is a tight scale on purpose — the shell lives between 11 and 13, and
/// anything larger belongs to onboarding and the welcome screen. Adding sizes
/// in between is how an interface stops looking like one thing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypeScale {
    /// 9px — badge counts, keycap hints.
    pub micro: f32,
    /// 10px — dense secondary labels.
    pub tiny: f32,
    /// 11px — paths, timestamps, status bar.
    pub small: f32,
    /// 12px — the default for interface text.
    pub body: f32,
    /// 13px — the editor buffer and terminal.
    pub code: f32,
    /// 14px — emphasised rows, section headers.
    pub large: f32,
    /// 15px — panel titles.
    pub title: f32,
    /// 20px — dialog headings.
    pub heading: f32,
    /// 24px — onboarding step headings.
    pub subdisplay: f32,
    /// 32px — the welcome wordmark.
    pub display: f32,
}

impl TypeScale {
    pub const fn default_scale() -> Self {
        Self {
            micro: 9.0,
            tiny: 10.0,
            small: 11.0,
            body: 12.0,
            code: 13.0,
            large: 14.0,
            title: 15.0,
            heading: 20.0,
            subdisplay: 24.0,
            display: 32.0,
        }
    }

    /// Every size multiplied — how the "font size" setting and OS display
    /// scaling apply without the proportions drifting.
    pub fn scaled(self, factor: f32) -> Self {
        Self {
            micro: self.micro * factor,
            tiny: self.tiny * factor,
            small: self.small * factor,
            body: self.body * factor,
            code: self.code * factor,
            large: self.large * factor,
            title: self.title * factor,
            heading: self.heading * factor,
            subdisplay: self.subdisplay * factor,
            display: self.display * factor,
        }
    }
}

impl Default for TypeScale {
    fn default() -> Self {
        Self::default_scale()
    }
}

/// Line height as a multiple of font size. Code needs more room than prose at
/// the same size, because the gutter and diff marks have to line up with it.
pub const LINE_HEIGHT_UI: f32 = 1.4;
pub const LINE_HEIGHT_CODE: f32 = 1.55;

// Enforced at compile time rather than in a test: if code ever gets tighter
// leading than interface text, the gutter and the diff marks stop lining up
// with the lines they belong to.
const _: () = assert!(LINE_HEIGHT_CODE > LINE_HEIGHT_UI);
