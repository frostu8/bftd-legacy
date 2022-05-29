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

use ggez::{Context, timer};
use ggez::event::EventHandler;
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::graphics;

use battle::{Battle, FRAMES_PER_SECOND};
use input::Inputs;
use assets::Bundle;

use anyhow::Error;

/// The main game state.
pub struct Game {
    core_bundle: Bundle,
    battle: Battle,
    sampler: sampler::Keyboard,
}

impl Game {
    /// Creates the main game state.
    pub fn new(cx: &mut Context) -> Result<Game, Error> {
        const ELEVATION: f32 = 100.0;

        let mut core_bundle = Bundle::new("./assets/")?;

        let gdfsm = core_bundle.load_character(cx, "/characters/grand_dad.ron").unwrap();

        let rect = ggez::graphics::Rect {
            x: -960.0,
            y: -1080.0 + ELEVATION,
            w: 1920.0,
            h: 1080.0,
        };
        graphics::set_screen_coordinates(cx, rect).unwrap();

        Ok(Game {
            core_bundle,
            battle: Battle::new(gdfsm.clone(), gdfsm),
            sampler: sampler::Keyboard::new(Default::default()),
        })
    }
}

impl EventHandler for Game {
    fn key_down_event(
        &mut self,
        ctx: &mut Context,
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
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
    ) {
        self.sampler.key_up(keycode);
    }

    fn update(&mut self, cx: &mut Context) -> ggez::GameResult {
        while timer::check_update_time(cx, FRAMES_PER_SECOND) {
            self.battle.update(self.sampler.sample(), Inputs::default()).unwrap();
        }

        Ok(())
    }

    fn draw(&mut self, cx: &mut Context) -> ggez::GameResult {
        graphics::clear(cx, [0.0, 0.0, 0.0, 1.0].into());

        self.battle.draw(cx).unwrap();

        graphics::present(cx)
    }
}


