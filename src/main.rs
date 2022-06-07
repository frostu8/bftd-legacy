use anyhow::Error;

use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use glam::f32::{Affine2, Vec2};

use bftd::render::{Drawable, Context, Sprite};

pub fn main() -> Result<(), Error> {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_min_inner_size(LogicalSize::new(0, 100))
        .build(&event_loop)
        .unwrap();

    let mut cx = Context::new(&window)?;

    let tex = cx.load_texture(std::fs::File::open("assets/img/grand_dad/idle.png").unwrap()).unwrap();
    let mut sprite: Sprite = tex.into();
    sprite.set_src(bftd_lib::Rect::new(0.25, 0., 0.75, 1.));
    sprite.set_transform(Affine2::from_translation(Vec2::new(0., -0.5)));

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                cx.resize(size.width, size.height);
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                cx.begin(|cx| {
                    cx.set_transform(Affine2::from_scale(Vec2::new(0.5, 0.5)));
                    sprite.draw(cx);
                });
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}

