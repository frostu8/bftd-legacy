use anyhow::Error;

use winit::{
    dpi::LogicalSize,
    event::{DeviceEvent, ElementState, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use bftd::Context;

pub fn main() -> Result<(), Error> {
    env_logger::init();

    let args = bftd::config::Args::from_args();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_min_inner_size(LogicalSize::new(0, 100))
        .build(&event_loop)
        .unwrap();

    let mut cx = Context {
        render: bftd::render::Context::new(&window)?,
        script: bftd::battle::script::Engine::new(),
        input: bftd::input::Sampler::new(Default::default()),
        frame_limiter: bftd::timer::FrameLimiter::new(),
        task_pool: bevy_tasks::TaskPool::new(),
        args,
    };

    let mut game = bftd::Game::new(&mut cx)?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                cx.render.resize(size.width, size.height);
                window.request_redraw();
            }
            Event::DeviceEvent {
                event: DeviceEvent::Key(key),
                ..
            } => match key.state {
                ElementState::Pressed => cx.input.process_key_down(key.scancode),
                ElementState::Released => cx.input.process_key_up(key.scancode),
            },
            Event::RedrawRequested(_) => {
                cx.render.begin(|mut cx| {
                    // set camera transform
                    // TODO: move this somewhere that makes sense
                    //cx.set_transform(Affine2::from_scale(Vec2::new(1. / 500., 1. / 500.)));
                    game.draw(&mut cx);
                });
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => {
                game.update(&mut cx);
                window.request_redraw();
            }
            _ => {}
        }
    });
}
