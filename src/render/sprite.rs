//! Sprite renderer.

use super::{Target, Texture, Drawable, Renderer};

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
use glam::f32::{Affine2, Vec2};
use bftd_lib::Rect;

use std::mem;

use bytemuck::{Pod, Zeroable};

/// Sprite pipeline layout.
pub struct Layout {
    bind_group_layout: BindGroupLayout,

    layout: PipelineLayout,
    pipeline: RenderPipeline,
    clip: Affine2,
}

impl Layout {
    /// Creates a new `Layout`.
    pub fn new(Target { device, surface_config, .. }: &Target) -> Layout {
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

        Layout {
            bind_group_layout,
            layout,
            pipeline,
            clip: get_clip_transform(surface_config),
        }
    }

    /// Reconfigures the layout.
    pub fn reconfigure(&mut self, Target { surface_config, .. }: &Target) {
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
    let norm_width = config.width as f32 / config.height as f32;
    
    Affine2::from_scale(Vec2::new(1.0, norm_width))
        * Affine2::from_scale(Vec2::new(0.5, 0.5))
        * Affine2::from_translation(Vec2::new(1.0, 1.0))
}

/// A sprite to be rendered to the screen.
pub struct Sprite<'a> {
    pub texture: &'a Texture,
    pub src: Rect,
    pub transform: Affine2,
}

impl<'a> Drawable for Sprite<'a> {
    fn draw(&self, renderer: &mut Renderer) {
        // normalize width
        let x = self.texture.width() as f32 / self.texture.height() as f32;

        // create buffer
        let vertex_data = [
            // bottom-left
            vertex(Vec2::ZERO, Vec2::new(self.src.left(), self.src.bottom())),
            // bottom-right
            vertex(Vec2::new(x, 0.), Vec2::new(self.src.right(), self.src.bottom())),
            // top-right
            vertex(Vec2::new(x, 1.), Vec2::new(self.src.right(), self.src.top())),
            // top-left
            vertex(Vec2::Y, Vec2::new(self.src.left(), self.src.top())),
        ];
        let vertex_buf = renderer.target.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: BufferUsages::VERTEX,
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
        rpass.set_vertex_buffer(0, vertex_buf.slice(..));
        rpass.pop_debug_group();
        rpass.insert_debug_marker("Draw!");
        rpass.draw(0..4, 0..1);
    }
}

