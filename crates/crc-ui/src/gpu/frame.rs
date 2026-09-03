use crc_theme::Rgba;

use crate::gpu::quad::Quad;
use crate::gpu::text::TextRun;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Layer {
    pub quads: Vec<Quad>,
    pub text: Vec<TextRun>,
}

impl Layer {
    pub fn is_empty(&self) -> bool {
        self.quads.is_empty() && self.text.is_empty()
    }

    pub fn clear(&mut self) {
        self.quads.clear();
        self.text.clear();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub background: Rgba,
    pub shell: Layer,
    pub overlay: Layer,
}

impl Frame {
    pub fn new(background: Rgba) -> Self {
        Self {
            background,
            shell: Layer::default(),
            overlay: Layer::default(),
        }
    }

    pub fn quad(&mut self, quad: Quad) -> &mut Self {
        self.shell.quads.push(quad);
        self
    }

    pub fn text(&mut self, run: TextRun) -> &mut Self {
        self.shell.text.push(run);
        self
    }

    pub fn overlay_quad(&mut self, quad: Quad) -> &mut Self {
        self.overlay.quads.push(quad);
        self
    }

    pub fn overlay_text(&mut self, run: TextRun) -> &mut Self {
        self.overlay.text.push(run);
        self
    }

    pub fn with_quads(mut self, quads: impl IntoIterator<Item = Quad>) -> Self {
        self.shell.quads.extend(quads);
        self
    }

    pub fn with_text(mut self, runs: impl IntoIterator<Item = TextRun>) -> Self {
        self.shell.text.extend(runs);
        self
    }

    pub fn quads(&self) -> &[Quad] {
        &self.shell.quads
    }

    pub fn runs(&self) -> &[TextRun] {
        &self.shell.text
    }

    pub fn clear(&mut self) {
        self.shell.clear();
        self.overlay.clear();
    }
}
