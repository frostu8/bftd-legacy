//! # `bftd`

#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate log;

pub mod assets;
pub mod battle;
pub mod fsm;
pub mod input;
pub mod sampler;

use ggez::timer;
use ggez::event::EventHandler;
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::graphics;

use std::ops::{Deref, DerefMut};

use battle::{Arena, FRAMES_PER_SECOND, script::Engine};
use input::Inputs;
use assets::Bundle;

use anyhow::Error;

/// The main game state.
pub struct Game {
    core_bundle: Bundle,
    battle: Arena,
    script_engine: Engine,
    sampler: sampler::Keyboard,
}

impl Game {
    /// Creates the main game state.
    pub fn new(cx: &mut ggez::Context) -> Result<Game, Error> {
        const ELEVATION: f32 = 100.0;

        let script_engine = Engine::new();
        let mut core_bundle = Bundle::new("./assets/")?;
        let mut cx = Context::new(cx, &script_engine);

        let gdfsm = core_bundle.load_character(&mut cx, "/characters/grand_dad.ron").unwrap();
        let hhfsm = core_bundle.load_character(&mut cx, "/characters/hh.ron").unwrap();

        let rect = ggez::graphics::Rect {
            x: -960.0,
            y: -1080.0 + ELEVATION,
            w: 1920.0,
            h: 1080.0,
        };
        graphics::set_screen_coordinates(&mut cx, rect).unwrap();

        Ok(Game {
            core_bundle,
            battle: Arena::new(gdfsm, hhfsm),
            script_engine,
            sampler: sampler::Keyboard::new(Default::default()),
        })
    }
}

impl EventHandler for Game {
    fn key_down_event(
        &mut self,
        ctx: &mut ggez::Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        repeat: bool
    ) {
        if keycode == KeyCode::Escape {
            ggez::event::quit(ctx);
            return;
        }

        if !repeat {
            self.sampler.key_down(keycode);
        }
    }

    fn key_up_event(
        &mut self,
        _ctx: &mut ggez::Context,
        keycode: KeyCode,
        _keymods: KeyMods,
    ) {
        self.sampler.key_up(keycode);
    }

    fn update(&mut self, cx: &mut ggez::Context) -> ggez::GameResult {
        while timer::check_update_time(cx, FRAMES_PER_SECOND) {
            let mut cx = Context::new(cx, &self.script_engine);
            self.battle.update(&mut cx, self.sampler.sample(), Inputs::default()).unwrap();
        }

        Ok(())
    }

    fn draw(&mut self, cx: &mut ggez::Context) -> ggez::GameResult {
        graphics::clear(cx, [0.0, 0.0, 0.0, 1.0].into());

        {
            let mut cx = Context::new(cx, &self.script_engine);
            self.battle.draw(&mut cx).unwrap();
        }

        graphics::present(cx)
    }
}

/// A game context.
pub struct Context<'a> {
    ggez: &'a mut ggez::Context,
    script_engine: &'a Engine,
}

impl<'a> Context<'a> {
    pub fn new(
        ggez: &'a mut ggez::Context,
        script_engine: &'a Engine,
    ) -> Context<'a> {
        Context { ggez, script_engine }
    }
}

impl<'a> Deref for Context<'a> {
    type Target = ggez::Context;

    fn deref(&self) -> &Self::Target {
        &self.ggez
    }
}

impl<'a> DerefMut for Context<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ggez
    }
}

