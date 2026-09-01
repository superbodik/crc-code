use crc_theme::Rgba;

use crate::gpu::quad::Quad;
use crate::gpu::text::TextRun;

#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub background: Rgba,
    pub quads: Vec<Quad>,
    pub text: Vec<TextRun>,
}

impl Frame {
    pub fn new(background: Rgba) -> Self {
        Self {
            background,
            quads: Vec::new(),
            text: Vec::new(),
        }
    }

    pub fn quad(&mut self, quad: Quad) -> &mut Self {
        self.quads.push(quad);
        self
    }

    pub fn text(&mut self, run: TextRun) -> &mut Self {
        self.text.push(run);
        self
    }

    pub fn with_quads(mut self, quads: impl IntoIterator<Item = Quad>) -> Self {
        self.quads.extend(quads);
        self
    }

    pub fn with_text(mut self, runs: impl IntoIterator<Item = TextRun>) -> Self {
        self.text.extend(runs);
        self
    }

    pub fn clear(&mut self) {
        self.quads.clear();
        self.text.clear();
    }
}
