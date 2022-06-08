//! # `bftd`

#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate log;

pub mod render;
pub mod assets;
pub mod battle;
pub mod input;
pub mod timer;

use input::Handle;
use render::Renderer;

use anyhow::Error;

/// Global game context.
pub struct Context {
    /// The render context.
    pub render: render::Context,
    /// The scripting engine used to run scripts in-battle.
    pub script: battle::script::Engine,
    /// The input handler.
    pub input: input::Sampler,
    /// A frame limiter.
    pub frame_limiter: timer::FrameLimiter,
    /// A thread pool for I/O.
    pub task_pool: bevy_tasks::TaskPool,
}

/// The game.
pub struct Game {
    core_bundle: assets::Bundle,
    battle: battle::LocalBattle,
}

impl Game {
    /// Creates a new game.
    pub fn new(cx: &mut Context) -> Result<Game, Error> {
        let mut core_bundle = assets::Bundle::new("assets/")?;

        let gdfsm = core_bundle.load_character(cx, "/characters/grand_dad.ron")?;
        let hhfsm = core_bundle.load_character(cx, "/characters/hh.ron")?;

        let arena = battle::Arena::new(&cx.script, gdfsm, hhfsm)?;

        Ok(Game {
            core_bundle,
            battle: battle::LocalBattle::new(arena, Handle::new(0), Handle::new(1)),
        })
    }

    /// Updates the game state.
    ///
    /// This should be called as frequently as possible.
    pub fn update(&mut self, cx: &mut Context) {
        // update input
        cx.input.poll();

        self.battle.update(cx).unwrap();
    }

    /// Draws the game state to the screen.
    pub fn draw(&mut self, cx: &mut Renderer) {
        self.battle.draw(cx).unwrap();
    }
}

