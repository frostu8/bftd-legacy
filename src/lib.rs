//! # `bftd`

#![feature(maybe_uninit_uninit_array, maybe_uninit_array_assume_init)]

#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate log;

pub mod assets;
pub mod battle;
pub mod config;
pub mod input;
pub mod render;
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
    /// Arguments passed to the application.
    pub args: config::Args,
}

/// The game.
pub struct Game {
    core_bundle: assets::Bundle,
    battle: battle::NetBattle,
}

impl Game {
    /// Creates a new game.
    pub fn new(cx: &mut Context) -> Result<Game, Error> {
        let mut core_bundle = assets::Bundle::new("assets/")?;

        let gdfsm = core_bundle.load_character(cx, "/characters/grand_dad.ron")?;
        let hhfsm = core_bundle.load_character(cx, "/characters/hh.ron")?;

        // note that arena is being made the same exact way
        let arena = battle::Arena::new(&cx.script, gdfsm, hhfsm)?;

        let p1: std::net::SocketAddr = ([127, 0, 0, 1], 19191).into();
        let p2: std::net::SocketAddr = ([127, 0, 0, 1], 19192).into();

        let battle = if cx.args.netmode == 0 {
            battle::NetBattle::new(cx, arena, p1, &[battle::NetPlayer::Local(Handle::new(0)), battle::NetPlayer::Remote(p2)])?
        } else if cx.args.netmode == 1 {
            battle::NetBattle::new(cx, arena, p2, &[battle::NetPlayer::Remote(p1), battle::NetPlayer::Local(Handle::new(0))])?
        } else {
            todo!()
        };

        Ok(Game {
            core_bundle,
            battle,
            //battle: battle::LocalBattle::new(arena, Handle::new(0), Handle::new(1)),
        })
    }

    /// Updates the game state.
    ///
    /// This should be called as frequently as possible.
    pub fn update(&mut self, cx: &mut Context) {
        self.battle.update(cx).unwrap();
    }

    /// Draws the game state to the screen.
    pub fn draw(&mut self, cx: &mut Renderer) {
        self.battle.draw(cx).unwrap();
    }
}

