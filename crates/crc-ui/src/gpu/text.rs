use std::ops::Range;

use crc_theme::{Rgba, Weight, typography};
use glyphon::cosmic_text::Align;
use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache,
    TextArea, TextAtlas, TextBounds, Viewport,
};

use crate::error::{Result, UiError};
use crate::geometry::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontKind {
    Sans,
    Mono,
    Icon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlign {
    #[default]
    Start,
    End,
    Center,
}

impl TextAlign {
    fn to_cosmic(self) -> Option<Align> {
        match self {
            TextAlign::Start => None,
            TextAlign::End => Some(Align::Right),
            TextAlign::Center => Some(Align::Center),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub range: Range<usize>,
    pub color: Rgba,
}

impl Span {
    pub fn new(range: Range<usize>, color: Rgba) -> Self {
        Self { range, color }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextRun {
    pub text: String,
    pub rect: Rect,
    pub size: f32,
    pub line_height: f32,
    pub color: Rgba,
    pub font: FontKind,
    pub weight: Weight,
    pub align: TextAlign,
    pub spans: Vec<Span>,
}

impl TextRun {
    pub fn new(text: impl Into<String>, rect: Rect, size: f32, color: Rgba) -> Self {
        Self {
            text: text.into(),
            rect,
            size,
            line_height: size * typography::LINE_HEIGHT_UI,
            color,
            font: FontKind::Sans,
            weight: Weight::Regular,
            align: TextAlign::Start,
            spans: Vec::new(),
        }
    }

    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    pub fn mono(mut self) -> Self {
        self.font = FontKind::Mono;
        self.line_height = self.size * typography::LINE_HEIGHT_CODE;
        self
    }

    pub fn icon(glyph: char, rect: Rect, size: f32, color: Rgba) -> Self {
        Self {
            font: FontKind::Icon,
            align: TextAlign::Center,
            line_height: size,
            ..Self::new(glyph.to_string(), rect, size, color)
        }
    }

    pub fn weight(mut self, weight: Weight) -> Self {
        self.weight = weight;
        self
    }

    pub fn line_height(mut self, line_height: f32) -> Self {
        self.line_height = line_height;
        self
    }

    pub fn spans(mut self, spans: Vec<Span>) -> Self {
        self.spans = spans;
        self
    }
}

pub fn segments<'a>(text: &'a str, spans: &[Span], default: Rgba) -> Vec<(&'a str, Rgba)> {
    let mut out = Vec::new();
    let mut cursor = 0usize;

    for span in spans {
        let start = floor_boundary(text, span.range.start).max(cursor);
        let end = floor_boundary(text, span.range.end).max(start);
        if start > cursor {
            out.push((&text[cursor..start], default));
        }
        if end > start {
            out.push((&text[start..end], span.color));
        }
        cursor = end;
    }

    if cursor < text.len() {
        out.push((&text[cursor..], default));
    }
    out
}

fn floor_boundary(text: &str, index: usize) -> usize {
    let mut index = index.min(text.len());
    while index > 0 && !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}

pub struct TextLayer {
    font_system: FontSystem,
    swash: SwashCache,
    atlas: TextAtlas,
    viewport: Viewport,
    renderer: glyphon::TextRenderer,
    buffers: Vec<Buffer>,
    areas: usize,
    sans: String,
    mono: String,
}

impl TextLayer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let mut font_system = FontSystem::new();
        font_system
            .db_mut()
            .load_font_data(crate::icon::DATA.to_vec());
        let cache = Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, format);
        let renderer =
            glyphon::TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);

        let sans = resolve(&font_system, typography::SANS);
        let mono = resolve(&font_system, typography::MONO);

        Self {
            font_system,
            swash: SwashCache::new(),
            atlas,
            viewport,
            renderer,
            buffers: Vec::new(),
            areas: 0,
            sans,
            mono,
        }
    }

    pub fn fonts(&self) -> (&str, &str) {
        (&self.sans, &self.mono)
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen: (u32, u32),
        runs: &[TextRun],
    ) -> Result<()> {
        self.viewport.update(
            queue,
            Resolution {
                width: screen.0,
                height: screen.1,
            },
        );

        while self.buffers.len() < runs.len() {
            self.buffers
                .push(Buffer::new(&mut self.font_system, Metrics::new(12.0, 16.0)));
        }
        self.areas = runs.len();

        for (buffer, run) in self.buffers.iter_mut().zip(runs) {
            let family = match run.font {
                FontKind::Sans => self.sans.as_str(),
                FontKind::Mono => self.mono.as_str(),
                FontKind::Icon => crate::icon::FAMILY,
            };
            let attrs = Attrs::new()
                .family(Family::Name(family))
                .weight(glyphon::Weight(run.weight as u16))
                .color(color_of(run.color));

            buffer.set_metrics(Metrics::new(run.size, run.line_height));
            buffer.set_size(Some(run.rect.width), Some(run.rect.height));

            let align = run.align.to_cosmic();
            if run.spans.is_empty() {
                buffer.set_text(&run.text, &attrs, Shaping::Advanced, align);
            } else {
                let pieces: Vec<(&str, Attrs)> = segments(&run.text, &run.spans, run.color)
                    .into_iter()
                    .map(|(piece, color)| (piece, attrs.clone().color(color_of(color))))
                    .collect();
                buffer.set_rich_text(pieces, &attrs, Shaping::Advanced, align);
            }

            buffer.shape_until_scroll(&mut self.font_system, false);
        }

        let areas = self.buffers.iter().zip(runs).map(|(buffer, run)| TextArea {
            buffer,
            left: run.rect.x,
            top: run.rect.y,
            scale: 1.0,
            bounds: TextBounds {
                left: run.rect.x as i32,
                top: run.rect.y as i32,
                right: run.rect.right().ceil() as i32,
                bottom: run.rect.bottom().ceil() as i32,
            },
            default_color: color_of(run.color),
            custom_glyphs: &[],
        });

        self.renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                areas,
                &mut self.swash,
            )
            .map_err(UiError::TextPrepare)
    }

    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) -> Result<()> {
        if self.areas == 0 {
            return Ok(());
        }
        self.renderer
            .render(&self.atlas, &self.viewport, pass)
            .map_err(UiError::TextRender)
    }

    pub fn measure(&mut self, run: &TextRun) -> (f32, f32) {
        let mut buffer = Buffer::new(
            &mut self.font_system,
            Metrics::new(run.size, run.line_height),
        );
        let family = match run.font {
            FontKind::Sans => self.sans.as_str(),
            FontKind::Mono => self.mono.as_str(),
            FontKind::Icon => crate::icon::FAMILY,
        };
        let attrs = Attrs::new()
            .family(Family::Name(family))
            .weight(glyphon::Weight(run.weight as u16));

        buffer.set_size(None, None);
        buffer.set_text(&run.text, &attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(&mut self.font_system, false);

        let mut width: f32 = 0.0;
        let mut lines = 0usize;
        for line in buffer.layout_runs() {
            width = width.max(line.line_w);
            lines += 1;
        }
        (width, lines as f32 * run.line_height)
    }
}

fn resolve(font_system: &FontSystem, family: typography::FontFamily) -> String {
    let database = font_system.db();
    let installed = |name: &str| {
        database
            .faces()
            .any(|face| face.families.iter().any(|(family, _)| family == name))
    };

    if installed(family.primary) {
        return family.primary.to_string();
    }
    for fallback in family.fallbacks {
        if installed(fallback) {
            return fallback.to_string();
        }
    }
    family
        .fallbacks
        .last()
        .unwrap_or(&family.primary)
        .to_string()
}

fn color_of(color: Rgba) -> Color {
    Color::rgba(color.r, color.g, color.b, color.a)
}
