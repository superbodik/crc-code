use crate::chrome::Chrome;
use crate::density::{Affordances, Density, Metrics};
use crate::diff::DiffTheme;
use crate::syntax::SyntaxTheme;
use crate::typography::TypeScale;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    pub chrome: Chrome,
    pub syntax: SyntaxTheme,
    pub diff: DiffTheme,
    pub type_scale: TypeScale,
    pub density: Density,
    pub zen: bool,
}

impl Theme {
    pub fn light() -> Self {
        Self {
            chrome: Chrome::light(),
            syntax: SyntaxTheme::light(),
            diff: DiffTheme::light(),
            type_scale: TypeScale::default_scale(),
            density: Density::Balanced,
            zen: false,
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

    pub fn metrics(&self) -> Metrics {
        self.density.metrics()
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
