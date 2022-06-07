use anyhow::Error;

use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

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
    let sprite = Sprite::new(tex);

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

