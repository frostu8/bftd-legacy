//! Sprite renderer.

use super::{Texture, Drawable, Renderer};

use wgpu::{
    include_wgsl, PipelineLayoutDescriptor, RenderPipelineDescriptor,
    VertexState, FragmentState, PrimitiveState, MultisampleState,
    PipelineLayout, RenderPipeline, SurfaceConfiguration,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, ShaderStages, BindingType,
    BufferBindingType, BufferSize, BindGroupLayout, VertexBufferLayout,
    BufferAddress, VertexAttribute, VertexStepMode, VertexFormat, util::DeviceExt,
    BufferUsages, RenderPassDescriptor, RenderPassColorAttachment, Operations,
    LoadOp,
};
use glam::f32::{Affine2, Mat3, Mat4, Vec2};
use bftd_lib::Rect;

use std::mem;

use bytemuck::{Pod, Zeroable};

/// Sprite pipeline layout.
pub struct Layout {
    bind_group_layout: BindGroupLayout,
    layout: PipelineLayout,
    pipeline: RenderPipeline,

    sampler: wgpu::Sampler,
    clip: Affine2,
}

impl Layout {
    /// Creates a new `Layout`.
    pub fn new(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> Layout {
        let vertex_size = mem::size_of::<Vertex>();

        let shader = device.create_shader_module(&include_wgsl!("sprite.wgsl"));

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("sprite shader bind group layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(64),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float {
                            filterable: true,
                        },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                }
            ],
        });
        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("sprite shader layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let vertex_buffer_layout = [VertexBufferLayout {
            array_stride: vertex_size as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    format: VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x2,
                    offset: 2 * 4,
                    shader_location: 1,
                },
            ],
        }];

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &vertex_buffer_layout,
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[surface_config.format.into()],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
        });

        // create default sampler
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Layout {
            bind_group_layout,
            layout,
            pipeline,

            sampler,
            clip: get_clip_transform(surface_config),
        }
    }

    /// Reconfigures the layout.
    pub fn reconfigure(&mut self, surface_config: &wgpu::SurfaceConfiguration) {
        self.clip = get_clip_transform(surface_config);
    }
}

#[derive(Copy, Clone, Debug, Pod, Zeroable)]
#[repr(C)]
struct Vertex {
    _pos: [f32; 2],
    _tex_coord: [f32; 2],
}

fn vertex(pos: Vec2, tc: Vec2) -> Vertex {
    Vertex {
        _pos: [pos.x, pos.y],
        _tex_coord: [tc.x, tc.y],
    }
}

fn get_clip_transform(config: &SurfaceConfiguration) -> Affine2 {
    // Our clip matrix aligns (0, 0) to the bottom left corner of the screen.
    // It also normalizes the dimensions of the graphics space so that it is
    // 1.0 unit tall.
    let norm_width = config.height as f32 / config.width as f32;
    
    Affine2::from_scale(Vec2::new(1., -1.))
        * Affine2::from_translation(-Vec2::new(0.0, 1.0))
        * Affine2::from_scale(Vec2::new(norm_width, 1.0))
        * Affine2::from_scale(Vec2::new(2.0, 2.0))
}

/// A sprite to be rendered to the screen.
pub struct Sprite {
    pub texture: Texture,
    pub src: Rect,
    pub transform: Affine2,
}

impl Sprite {
    /// Creates a new sprite.
    pub fn new(texture: Texture) -> Sprite {
        Sprite {
            texture,
            src: Rect { p1: Vec2::ZERO, p2: Vec2::ONE },
            transform: Default::default(),
        }
    }
}

impl Drawable for Sprite {
    fn draw(&self, renderer: &mut Renderer) {
        // normalize width
        let x = (self.src.width() * self.texture.width() as f32) / (self.src.height() * self.texture.height() as f32);

        // create vertex buffer
        let vertex_data = [
            // bottom-left
            vertex(Vec2::ZERO, Vec2::new(self.src.left(), self.src.bottom())),
            // bottom-right
            vertex(Vec2::new(x, 0.), Vec2::new(self.src.right(), self.src.bottom())),
            // top-right
            vertex(Vec2::new(x, 1.), Vec2::new(self.src.right(), self.src.top())),
            // top-left
            vertex(Vec2::new(0., 1.), Vec2::new(self.src.left(), self.src.top())),
        ];
        let vertex_buf = renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: BufferUsages::VERTEX,
        });

        // create index buffer
        let index_data: [u16; 6] = [0, 1, 2, 2, 3, 0];
        let index_buf = renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&index_data),
            usage: wgpu::BufferUsages::INDEX,
        });

        // create transform matrix
        let mx = Mat3::from(self.transform * renderer.sprite.clip);
        let mx = Mat4::from_mat3(mx);
        let mx_ref: &[f32; 16] = mx.as_ref();
        let uniform_buf = renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("transform uniform"),
            contents: bytemuck::cast_slice(mx_ref),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let texture_view = self.texture.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("texture"),
            ..Default::default()
        });

        let bind_group = renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &renderer.sprite.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&renderer.sprite.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
            ],
            label: None,
        });

        let mut rpass = renderer.encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[RenderPassColorAttachment {
                view: &renderer.view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
        rpass.push_debug_group("Prepare data for draw.");
        rpass.set_pipeline(&renderer.cx.sprite.pipeline);
        rpass.set_bind_group(0, &bind_group, &[]);
        rpass.set_index_buffer(index_buf.slice(..), wgpu::IndexFormat::Uint16);
        rpass.set_vertex_buffer(0, vertex_buf.slice(..));
        rpass.pop_debug_group();
        rpass.insert_debug_marker("Draw!");
        rpass.draw_indexed(0..6, 0, 0..1);
    }
}

