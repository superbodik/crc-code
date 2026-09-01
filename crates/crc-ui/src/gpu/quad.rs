use crc_theme::Rgba;
use wgpu::util::DeviceExt;

use crate::geometry::Rect;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quad {
    pub rect: Rect,
    pub color: Rgba,
    pub border_color: Rgba,
    pub border_width: f32,
    pub radius: f32,
}

impl Quad {
    pub fn filled(rect: Rect, color: Rgba) -> Self {
        Self {
            rect,
            color,
            border_color: Rgba::TRANSPARENT,
            border_width: 0.0,
            radius: 0.0,
        }
    }

    pub fn rounded(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn bordered(mut self, width: f32, color: Rgba) -> Self {
        self.border_width = width;
        self.border_color = color;
        self
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    rect: [f32; 4],
    color: [f32; 4],
    border_color: [f32; 4],
    radius_border: [f32; 2],
    _padding: [f32; 2],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Globals {
    screen: [f32; 2],
    _padding: [f32; 2],
}

pub struct QuadRenderer {
    pipeline: wgpu::RenderPipeline,
    globals: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    instances: wgpu::Buffer,
    capacity: usize,
    count: u32,
}

impl QuadRenderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("quad"),
            source: wgpu::ShaderSource::Wgsl(include_str!("quad.wgsl").into()),
        });

        let globals = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad globals"),
            contents: bytemuck::bytes_of(&Globals {
                screen: [1.0, 1.0],
                _padding: [0.0; 2],
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("quad globals"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("quad globals"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("quad"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("quad"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Instance>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x4,
                        1 => Float32x4,
                        2 => Float32x4,
                        3 => Float32x2,
                    ],
                })],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            globals,
            bind_group,
            instances: empty_instance_buffer(device, 64),
            capacity: 64,
            count: 0,
        }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen: (f32, f32),
        quads: &[Quad],
    ) {
        queue.write_buffer(
            &self.globals,
            0,
            bytemuck::bytes_of(&Globals {
                screen: [screen.0.max(1.0), screen.1.max(1.0)],
                _padding: [0.0; 2],
            }),
        );

        let instances: Vec<Instance> = quads
            .iter()
            .filter(|quad| !quad.rect.is_empty())
            .map(|quad| Instance {
                rect: [quad.rect.x, quad.rect.y, quad.rect.width, quad.rect.height],
                color: quad.color.to_linear(),
                border_color: quad.border_color.to_linear(),
                radius_border: [quad.radius, quad.border_width],
                _padding: [0.0; 2],
            })
            .collect();

        self.count = instances.len() as u32;
        if instances.is_empty() {
            return;
        }

        if instances.len() > self.capacity {
            self.capacity = instances.len().next_power_of_two();
            self.instances = empty_instance_buffer(device, self.capacity);
        }
        queue.write_buffer(&self.instances, 0, bytemuck::cast_slice(&instances));
    }

    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        if self.count == 0 {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.instances.slice(..));
        pass.draw(0..6, 0..self.count);
    }
}

fn empty_instance_buffer(device: &wgpu::Device, capacity: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("quad instances"),
        size: (capacity * std::mem::size_of::<Instance>()) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
