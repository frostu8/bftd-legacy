//! Sprite renderer.

use super::{Texture, Drawable, Renderer};

use std::fmt::{self, Debug, Formatter};

use wgpu::util::DeviceExt;
use glam::f32::{Affine2, Mat3, Mat4, Vec2};
use bftd_lib::Rect;

/// Sprite shader.
pub struct Shader {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,

    sampler: wgpu::Sampler,
}

impl Shader {
    /// Creates a new `Shader`.
    pub fn new(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> Shader {
        let shader = device.create_shader_module(&wgpu::include_wgsl!("sprite.wgsl"));

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sprite shader bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float {
                            filterable: true,
                        },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(64),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(64),
                    },
                    count: None,
                },
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sprite shader layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
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

        Shader {
            bind_group_layout,
            pipeline,

            sampler,
        }
    }
}

/// A sprite to be rendered to the screen.
#[derive(Clone)]
pub struct Sprite {
    texture: Texture,
    src: Rect,
    transform: Affine2,
}

impl Sprite {
    /// Creates a new sprite, using the whole bounds of the texture as the src.
    pub fn new(texture: Texture) -> Sprite {
        let sprite = Sprite {
            texture,
            src: Rect { p1: Vec2::ZERO, p2: Vec2::ONE },
            transform: Default::default(),
        };

        sprite
    }

    /// The source rectangle of the sprite.
    pub fn src(&self) -> Rect {
        self.src.clone()
    }

    /// Sets the source rectangle of the sprite.
    pub fn set_src(&mut self, src: Rect) {
        self.src = src;
    }

    /// The transformation of the sprite.
    pub fn transform(&self) -> Affine2 {
        self.transform
    }

    /// Sets the transformation of the sprite.
    pub fn set_transform(&mut self, transform: Affine2) {
        self.transform = transform;
    }
}

impl Debug for Sprite {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f
            .debug_struct("Sprite")
            .field("src", &self.src)
            .field("transform", &self.transform)
            .finish_non_exhaustive()
    }
}

/// Converts a [`Texture`] to a sprite consisting of the entire bounds of the
/// texture.
impl From<Texture> for Sprite {
    fn from(texture: Texture) -> Sprite {
        Sprite::new(texture)
    }
}

impl Drawable for Sprite {
    fn draw(&self, renderer: &mut Renderer) {
        // normalize width
        let x = (self.src.width() * self.texture.width() as f32) / (self.src.height() * self.texture.height() as f32);

        // recreate transform matrix
        let transform = 
            renderer.clip
            * renderer.world
            * self.transform
            * Affine2::from_scale(Vec2::new(x, 1.0));
        let transform = Mat4::from_mat3(Mat3::from(transform));
        let transform_ref: &[f32; 16] = transform.as_ref();
        let transform = renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("transform uniform"),
            contents: bytemuck::cast_slice(transform_ref),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // do the same for the tex coord transform
        let tex_transform = Affine2::from_scale(Vec2::new(self.src.width(), self.src.height()))
            * Affine2::from_translation(Vec2::new(self.src.left(), self.src.bottom()));
        let tex_transform = Mat4::from_mat3(Mat3::from(tex_transform));
        let tex_transform_ref: &[f32; 16] = tex_transform.as_ref();
        let tex_transform = renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("tex transform uniform"),
            contents: bytemuck::cast_slice(tex_transform_ref),
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
                    resource: wgpu::BindingResource::Sampler(&renderer.sprite.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: transform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: tex_transform.as_entire_binding(),
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
        rpass.set_pipeline(&renderer.cx.sprite.pipeline);
        rpass.set_bind_group(0, &bind_group, &[]);
        rpass.draw(0..6, 0..1);
    }
}

