use crc_theme::Rgba;

use crate::error::Result;
use crate::gpu::{Gpu, Quad, QuadRenderer};

pub struct Offscreen {
    gpu: Gpu,
    quads: QuadRenderer,
    texture: wgpu::Texture,
    readback: wgpu::Buffer,
    width: u32,
    height: u32,
    padded_bytes_per_row: u32,
}

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

impl Offscreen {
    pub fn new(width: u32, height: u32) -> Result<Self> {
        let gpu = Gpu::headless()?;
        let quads = QuadRenderer::new(&gpu.device, FORMAT);

        let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("offscreen"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let unpadded = width * 4;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = unpadded.div_ceil(align) * align;

        let readback = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("offscreen readback"),
            size: (padded_bytes_per_row * height) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        Ok(Self {
            gpu,
            quads,
            texture,
            readback,
            width,
            height,
            padded_bytes_per_row,
        })
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn adapter(&self) -> String {
        self.gpu.describe()
    }

    pub fn render(&mut self, background: Rgba, quads: &[Quad]) -> Vec<u8> {
        self.quads.prepare(
            &self.gpu.device,
            &self.gpu.queue,
            (self.width as f32, self.height as f32),
            quads,
        );

        let view = self
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("offscreen"),
            });

        {
            let clear = background.to_linear();
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("offscreen"),
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
        }

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &self.readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(self.padded_bytes_per_row),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );

        self.gpu.queue.submit(Some(encoder.finish()));

        let slice = self.readback.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.gpu
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("the GPU finished the frame");

        let padded = slice
            .get_mapped_range()
            .expect("the frame is readable once the GPU is done");
        let mut pixels = Vec::with_capacity((self.width * self.height * 4) as usize);
        for row in 0..self.height {
            let start = (row * self.padded_bytes_per_row) as usize;
            pixels.extend_from_slice(&padded[start..start + (self.width * 4) as usize]);
        }
        drop(padded);
        self.readback.unmap();

        pixels
    }

    pub fn pixel(&self, frame: &[u8], x: u32, y: u32) -> Rgba {
        let index = ((y * self.width + x) * 4) as usize;
        Rgba::new(
            frame[index],
            frame[index + 1],
            frame[index + 2],
            frame[index + 3],
        )
    }
}
