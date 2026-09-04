use crate::chrome::Chrome;
use crate::density::{Affordances, Density, Metrics};
use crate::diff::DiffTheme;
use crate::syntax::SyntaxTheme;
use crate::typography::TypeScale;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Appearance {
    #[default]
    Light,
    Dark,
}

impl Appearance {
    pub const fn flipped(self) -> Self {
        match self {
            Appearance::Light => Appearance::Dark,
            Appearance::Dark => Appearance::Light,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Appearance::Light => "Светлая",
            Appearance::Dark => "Тёмная",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    pub appearance: Appearance,
    pub chrome: Chrome,
    pub syntax: SyntaxTheme,
    pub diff: DiffTheme,
    pub terminal: crate::terminal::TerminalTheme,
    pub type_scale: TypeScale,
    pub density: Density,
    pub scale: f32,
    pub zen: bool,
}

impl Theme {
    pub fn new(appearance: Appearance) -> Self {
        match appearance {
            Appearance::Light => Self::light(),
            Appearance::Dark => Self::dark(),
        }
    }

    pub fn light() -> Self {
        Self {
            appearance: Appearance::Light,
            chrome: Chrome::light(),
            syntax: SyntaxTheme::light(),
            diff: DiffTheme::light(),
            terminal: crate::terminal::TerminalTheme::light(),
            type_scale: TypeScale::default_scale(),
            density: Density::Balanced,
            scale: 1.0,
            zen: false,
        }
    }

    pub fn dark() -> Self {
        Self {
            appearance: Appearance::Dark,
            chrome: Chrome::dark(),
            syntax: SyntaxTheme::dark(),
            diff: DiffTheme::dark(),
            terminal: crate::terminal::TerminalTheme::dark(),
            type_scale: TypeScale::default_scale(),
            density: Density::Balanced,
            scale: 1.0,
            zen: false,
        }
    }

    pub fn with_appearance(self, appearance: Appearance) -> Self {
        Self {
            density: self.density,
            scale: self.scale,
            zen: self.zen,
            type_scale: self.type_scale,
            ..Self::new(appearance)
        }
    }

    pub fn with_density(mut self, density: Density) -> Self {
        self.density = density;
        self
    }

    pub fn with_code_size(mut self, size: f32) -> Self {
        let base = TypeScale::default_scale();
        self.type_scale = base.scaled(size / base.code);
        self
    }

    pub fn with_scale(mut self, factor: f32) -> Self {
        let factor = factor.max(0.1);
        self.type_scale = TypeScale::default_scale().scaled(factor);
        self.scale = factor;
        self
    }

    pub fn metrics(&self) -> Metrics {
        self.density.metrics().scaled(self.scale)
    }

    pub fn affordances(&self) -> Affordances {
        if self.zen {
            return Affordances {
                minimap: false,
                bottom_panel: false,
                inline_diagnostics: false,
                breadcrumbs: false,
            };
        }
        self.density.affordances()
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}
