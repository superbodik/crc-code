use std::sync::Arc;

use winit::window::Window;

use crate::error::{Result, UiError};
use crate::gpu::{Frame, Gpu, QuadRenderer, TextLayer, TextRun};

pub struct WindowRenderer {
    gpu: Gpu,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    quads: QuadRenderer,
    text: TextLayer,
    overlay_quads: QuadRenderer,
    overlay_text: TextLayer,
}

impl WindowRenderer {
    pub fn new(window: Arc<Window>) -> Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let surface = instance
            .create_surface(window.clone())
            .map_err(UiError::Surface)?;
        let gpu = pollster::block_on(Gpu::new(&instance, Some(&surface)))?;

        let size = window.inner_size();
        let capabilities = surface.get_capabilities(&gpu.adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(|format| format.is_srgb())
            .unwrap_or(capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            color_space: wgpu::SurfaceColorSpace::Auto,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&gpu.device, &config);

        let quads = QuadRenderer::new(&gpu.device, format);
        let text = TextLayer::new(&gpu.device, &gpu.queue, format);
        let overlay_quads = QuadRenderer::new(&gpu.device, format);
        let overlay_text = TextLayer::new(&gpu.device, &gpu.queue, format);

        Ok(Self {
            gpu,
            surface,
            config,
            quads,
            text,
            overlay_quads,
            overlay_text,
        })
    }

    pub fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    pub fn adapter(&self) -> String {
        self.gpu.describe()
    }

    pub fn fonts(&self) -> (&str, &str) {
        self.text.fonts()
    }

    pub fn measure(&mut self, run: &TextRun) -> (f32, f32) {
        self.text.measure(run)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.gpu.device, &self.config);
    }

    pub fn render(&mut self, frame: &Frame) -> Result<()> {
        use wgpu::CurrentSurfaceTexture;

        let surface_texture = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(texture)
            | CurrentSurfaceTexture::Suboptimal(texture) => texture,
            CurrentSurfaceTexture::Outdated | CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.gpu.device, &self.config);
                return Ok(());
            }
            other => {
                tracing::debug!("skipping a frame: {other:?}");
                return Ok(());
            }
        };

        let screen = (self.config.width as f32, self.config.height as f32);
        let pixels = (self.config.width, self.config.height);

        self.quads.prepare(
            &self.gpu.device,
            &self.gpu.queue,
            screen,
            &frame.shell.quads,
        );
        self.text
            .prepare(&self.gpu.device, &self.gpu.queue, pixels, &frame.shell.text)?;
        self.overlay_quads.prepare(
            &self.gpu.device,
            &self.gpu.queue,
            screen,
            &frame.overlay.quads,
        );
        self.overlay_text.prepare(
            &self.gpu.device,
            &self.gpu.queue,
            pixels,
            &frame.overlay.text,
        )?;

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame"),
            });

        {
            let clear = frame.background.to_linear();
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("frame"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: clear[0] as f64,
                            g: clear[1] as f64,
                            b: clear[2] as f64,
                            a: clear[3] as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            self.quads.draw(&mut pass);
            self.text.draw(&mut pass)?;
            self.overlay_quads.draw(&mut pass);
            self.overlay_text.draw(&mut pass)?;
        }

        self.gpu.queue.submit(Some(encoder.finish()));
        self.gpu.queue.present(surface_texture);
        Ok(())
    }
}
