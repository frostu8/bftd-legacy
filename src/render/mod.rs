//! 2D rendering using [`wgpu`].
//!
//! This also exposes [`wgpu`] types if you need to implement your own shaders,
//! for whatever reason.

mod sprite;

use pollster::FutureExt as _;

use wgpu::{
    Adapter, Backends, Instance, Device, Queue, TextureUsages, PresentMode,
    SurfaceConfiguration, TextureDescriptor, Extent3d, TextureFormat,
    TextureDimension, CommandEncoder, TextureView,
};
use winit::window::Window;

use std::ops::{Deref, DerefMut};

use anyhow::Error;

/// A surface.
pub struct Surface {
    target: Target,

    sprite: sprite::Layout,
}

impl Surface {
    /// Creates and initializes a `Surface`.
    ///
    /// **WARNING!** This is not meant to be called lightly. This will block
    /// while setting up and compiling shaders, which may take a while.
    pub fn new(window: &Window) -> Result<Surface, Error> {
        let size = window.inner_size();

        // create new instance
        let instance = Instance::new(Backends::all());

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
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Mailbox,
        };

        surface.configure(&device, &config);

        // build target
        let target = Target { adapter, device, queue, surface_config: config };

        Ok(Surface {
            // build the default render layouts
            sprite: sprite::Layout::new(&target),
            // finalize with target
            target,
        })
    }

    /// Begins a render frame, calls the closure and finalizes the frame.
    pub fn begin<F>(&mut self, f: F)
    where
        F: FnOnce(Renderer),
    {
    }

    /// Creates a 2D texture.
    pub fn create_texture(&self, width: u32, height: u32) -> Texture {
        let texture = self.target.device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Uint,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST,
        });

        Texture {
            texture,
            dims: (width, height),
        }
    }
}

/// A single frame to draw to.
pub struct Renderer<'a> {
    cx: &'a mut Surface,

    view: TextureView,
    encoder: CommandEncoder,
}

impl<'a> Deref for Renderer<'a> {
    type Target = Surface;

    fn deref(&self) -> &Surface {
        self.cx
    }
}

impl<'a> DerefMut for Renderer<'a> {
    fn deref_mut(&mut self) -> &mut Surface {
        self.cx
    }
}

/// A collection of [`wgpu`] structs used to execute graphics commands.
#[doc(hidden)]
pub struct Target {
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
    pub surface_config: SurfaceConfiguration,
}

/// A texture.
pub struct Texture {
    texture: wgpu::Texture,
    dims: (u32, u32),
}

impl Texture {
    /// Replaces the texture's data with data from a buffer.
    ///
    /// The function will do its best to decode the texture from magic numbers
    /// or encoding hints.
    pub fn load(&mut self, buf: &[u8]) -> Result<(), Error> {
        //let image = image::load_from_memory(buf);
        Ok(())
    }

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

