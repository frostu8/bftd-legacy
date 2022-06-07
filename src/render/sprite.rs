//! Sprite renderer.

use super::{Texture, Drawable, Renderer};

use wgpu::util::DeviceExt;
use glam::f32::{Affine2, Mat3, Mat4, Vec2};
use bftd_lib::Rect;

use std::mem;

use bytemuck::{Pod, Zeroable};

/// Sprite shader.
pub struct Shader {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,

    square_indices: wgpu::Buffer,
    sampler: wgpu::Sampler,

    clip: Affine2,
}

impl Shader {
    /// Creates a new `Shader`.
    pub fn new(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> Shader {
        let vertex_size = mem::size_of::<Vertex>();

        let shader = device.create_shader_module(&wgpu::include_wgsl!("sprite.wgsl"));

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sprite shader bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(64),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
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
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sprite shader layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let vertex_buffer_layout = [wgpu::VertexBufferLayout {
            array_stride: vertex_size as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 2 * 4,
                    shader_location: 1,
                },
            ],
        }];

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &vertex_buffer_layout,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // create render pipeline defaults
        // sampler
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

        // square_indices
        let square_indices_data: [u16; 6] = [0, 1, 2, 2, 3, 0];
        let square_indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&square_indices_data),
            usage: wgpu::BufferUsages::INDEX,
        });

        Shader {
            bind_group_layout,
            pipeline,

            sampler,
            square_indices,

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

fn get_clip_transform(config: &wgpu::SurfaceConfiguration) -> Affine2 {
    // Our clip matrix aligns (0, 0) to the bottom left corner of the screen.
    // It also normalizes the dimensions of the graphics space so that it is
    // 1.0 unit tall.
    let norm_width = config.height as f32 / config.width as f32;
    
    Affine2::from_scale(Vec2::new(norm_width, 1.0))
        * Affine2::from_scale(Vec2::new(2.0, 2.0))
        * Affine2::from_scale(Vec2::new(1., -1.))
}

/// A sprite to be rendered to the screen.
pub struct Sprite {
    pub texture: Texture,
    pub src: Rect,
    pub transform: Affine2,

    vertices: wgpu::Buffer,
}

impl Sprite {
    const MESH_SIZE: usize = mem::size_of::<[Vertex; 4]>();

    /// Creates a new sprite, using the whole bounds of the texture as the src.
    fn new(device: &wgpu::Device, texture: Texture) -> Sprite {
        let sprite = Sprite {
            texture,
            src: Rect { p1: Vec2::ZERO, p2: Vec2::ONE },
            transform: Default::default(),

            vertices: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("sprite mesh"),
                size: Self::MESH_SIZE as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::VERTEX,
                mapped_at_creation: true,
            }),
        };

        // rebuild meshes
        sprite.rebuild_mesh();

        sprite
    }

    fn rebuild_mesh(&self) {
        // get normalized width
        let x = (self.src.width() * self.texture.width() as f32)
            / (self.src.height() * self.texture.height() as f32);
        let half_x = x / 2.;

        // create vertex data
        let vertex_data = [
            // bottom-left
            vertex(Vec2::new(-half_x, -0.5), Vec2::new(self.src.left(), self.src.bottom())),
            // bottom-right
            vertex(Vec2::new(half_x, -0.5), Vec2::new(self.src.right(), self.src.bottom())),
            // top-right
            vertex(Vec2::new(half_x, 0.5), Vec2::new(self.src.right(), self.src.top())),
            // top-left
            vertex(Vec2::new(-half_x, 0.5), Vec2::new(self.src.left(), self.src.top())),
        ];
        self.vertices.slice(..).get_mapped_range_mut()[..Self::MESH_SIZE]
            .copy_from_slice(bytemuck::cast_slice(&vertex_data));
        self.vertices.unmap();
    }
}

/// Converts a [`Texture`] to a sprite consisting of the entire bounds of the
/// texture.
impl From<Texture> for Sprite {
    fn from(texture: Texture) -> Sprite {
        let device = texture.device.clone();

        Sprite::new(&device, texture)
    }
}

impl Drawable for Sprite {
    fn draw(&self, renderer: &mut Renderer) {
        // recreate transform matrix
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

        let mut rpass = renderer.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &renderer.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
        rpass.push_debug_group("Prepare data for draw.");
        rpass.set_pipeline(&renderer.cx.sprite.pipeline);
        rpass.set_bind_group(0, &bind_group, &[]);
        rpass.set_index_buffer(renderer.cx.sprite.square_indices.slice(..), wgpu::IndexFormat::Uint16);
        rpass.set_vertex_buffer(0, self.vertices.slice(..));
        rpass.pop_debug_group();
        rpass.insert_debug_marker("Draw!");
        rpass.draw_indexed(0..6, 0, 0..1);
    }
}

