use crate::chrome::Chrome;
use crate::density::{Affordances, Density, Metrics};
use crate::diff::DiffTheme;
use crate::syntax::SyntaxTheme;
use crate::typography::TypeScale;

/// Everything the renderer needs to draw the shell.
///
/// One value, passed down. A panel never reaches for a constant or decides its
/// own padding, which is what keeps the profiles honest — change the density
/// and every pane moves together.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    pub chrome: Chrome,
    pub syntax: SyntaxTheme,
    pub diff: DiffTheme,
    pub type_scale: TypeScale,
    pub density: Density,
    /// Zen (⌥Z): panels go away, the buffer stays.
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

    /// Apply the editor font-size setting (12, 13, 14, 16), scaling the rest of
    /// the scale with it so the shell keeps its proportions.
    pub fn with_code_size(mut self, size: f32) -> Self {
        let base = TypeScale::default_scale();
        self.type_scale = base.scaled(size / base.code);
        self
    }

    pub fn metrics(&self) -> Metrics {
        self.density.metrics()
    }

    /// What is on screen, after zen has had its say.
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
