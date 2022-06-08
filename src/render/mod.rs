//! 2D rendering using [`wgpu`].
//!
//! This also exposes [`wgpu`] types if you need to implement your own shaders,
//! for whatever reason.

mod sprite;

pub use sprite::Sprite;

use pollster::FutureExt as _;

use wgpu::util::DeviceExt;
use winit::window::Window;
use glam::f32::{Affine2, Vec2};

use std::ops::Deref;
use std::io::{Read, Seek, BufReader};
use std::sync::Arc;

use anyhow::Error;

/// A graphics context.
pub struct Context {
    device: Arc<wgpu::Device>,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,

    sprite: sprite::Shader,
}

impl Context {
    /// Creates and initializes a `Surface`.
    ///
    /// **WARNING!** This is not meant to be called lightly. This will block
    /// while setting up and compiling shaders, which may take a while.
    pub fn new(window: &Window) -> Result<Context, Error> {
        let size = window.inner_size();

        // create new instance
        let instance = wgpu::Instance::new(wgpu::Backends::all());

        // SAFETY: This is unsafe because the window handle must be valid, if
        // you find a way to have an invalid winit::Window then you have bigger
        // issues
        let surface = unsafe { instance.create_surface(window) };

        // find adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                // Request an adapter which can render to our surface
                compatible_surface: Some(&surface),
            })
            .block_on()
            .ok_or_else(|| anyhow!(ERR_NO_ADAPTER))?;

        // create logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                },
                None,
            )
            .block_on()?;

        // get swapchain format
        let swapchain_format = surface.get_preferred_format(&adapter).unwrap();

        // configure the surface
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        surface.configure(&device, &surface_config);

        Ok(Context {
            // build the default render layouts
            sprite: sprite::Shader::new(&device, &surface_config),
            // finalize
            device: Arc::new(device),
            queue,
            surface,
            surface_config,
        })
    }

    /// Resizes the swapchain texture.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
    }

    /// Begins a render frame, calls the closure and finalizes the frame.
    pub fn begin<F>(&self, f: F)
    where
        F: FnOnce(&mut Renderer),
    {
        // find clip transform
        // Our clip matrix aligns (0, 0) to the bottom left corner of the screen.
        // It also normalizes the dimensions of the graphics space so that it is
        // 1.0 unit tall.
        let norm_width = self.surface_config.height as f32 / self.surface_config.width as f32;
        
        let clip = Affine2::from_scale(Vec2::new(norm_width, 1.0))
            * Affine2::from_scale(Vec2::new(2.0, 2.0))
            * Affine2::from_scale(Vec2::new(1., -1.));

        // create swapchain view
        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            // clear screen
            let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
        }

        f(&mut Renderer {
            cx: self,
            
            world: Affine2::IDENTITY,
            clip,

            view,
            encoder: &mut encoder,
        });

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    /// Loads a 2D texture as an image from a stream.
    pub fn load_texture<R>(&self, read: R) -> Result<Texture, Error>
    where
        R: Read + Seek,
    {
        let image = image::io::Reader::new(BufReader::new(read))
            .with_guessed_format()?
            .decode()?
            .into_rgba8();

        let texture = self.device.create_texture_with_data(
            &self.queue,
            &wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: image.width(),
                    height: image.height(),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST,
            },
            image.as_raw(),
        );

        Ok(Texture {
            texture: Arc::new(texture),
            dims: (image.width(), image.height()),
        })
    }
}

/// A single frame to draw to.
pub struct Renderer<'a> {
    cx: &'a Context,

    // transformation matrices
    world: Affine2,
    clip: Affine2,

    view: wgpu::TextureView,
    encoder: &'a mut wgpu::CommandEncoder,
}

impl<'a> Renderer<'a> {
    /// The world transform.
    pub fn transform(&self) -> Affine2 {
        self.world
    }

    /// Sets the world transform.
    ///
    /// This is applied after clip transformations, but before the sprite
    /// transformations. A good place to put camera transforms.
    pub fn set_transform(&mut self, world: Affine2) {
        self.world = world;
    }
}

impl<'a> Deref for Renderer<'a> {
    type Target = Context;

    fn deref(&self) -> &Context {
        self.cx
    }
}

/// A texture.
///
/// Cheaply cloneable.
#[derive(Clone)]
pub struct Texture {
    texture: Arc<wgpu::Texture>,
    dims: (u32, u32),
}

impl Texture {
    /// The width of the texture.
    pub fn width(&self) -> u32 {
        self.dims.0
    }

    /// The height of the texture.
    pub fn height(&self) -> u32 {
        self.dims.1
    }
}

/// A trait for drawable items.
pub trait Drawable {
    /// Draws the item to the screen.
    fn draw(&self, renderer: &mut Renderer);
}

const ERR_NO_ADAPTER: &str = r#"
Cannot find a graphics adapter!

Check if your drivers are installed or up-to-date. bftd only supports Vulkan \
on linux, Direct3D on Windows and Metal on MacOSX."#;

