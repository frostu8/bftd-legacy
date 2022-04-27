use bftd::Game;

use ggez::conf::{FullscreenType, WindowMode};

use anyhow::Result;

pub fn main() -> Result<()> {
    env_logger::init();

    let (mut ctx, event_loop) = ggez::ContextBuilder::new("super_simple", "ggez")
        .window_mode(WindowMode {
            fullscreen_type: FullscreenType::Desktop,
            ..Default::default()
        })
        .build()?;

    let game = Game::new(&mut ctx)?;

    ggez::event::run(ctx, event_loop, game)
}

