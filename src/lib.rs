//! # `bftd`

#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate log;

pub mod assets;
pub mod battle;
pub mod fsm;
pub mod input;

use ggez::event::{Axis, Button, EventHandler};
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::input::gamepad::GamepadId;
use ggez::graphics;

use std::ops::{Deref, DerefMut};

use battle::{LocalBattle, Arena, script::Engine};
use input::sampler::{Input, Handle};
use assets::Bundle;

use anyhow::Error;

/// The main game state.
pub struct Game {
    core_bundle: Bundle,
    script_engine: Engine,
    input: Input,
    battle: LocalBattle,
}

impl Game {
    /// Creates the main game state.
    pub fn new(cx: &mut ggez::Context) -> Result<Game, Error> {
        const ELEVATION: f32 = 100.0;

        let script_engine = Engine::new();
        let mut core_bundle = Bundle::new("./assets/")?;
        let mut input = Input::new(cx);
        let mut cx = Context::new(cx, &script_engine, &mut input);

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
            battle: LocalBattle::new(Arena::new(cx.script_engine, gdfsm, hhfsm).unwrap(), Handle::new(0), Handle::new(1)),
            script_engine,
            input,
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
            self.input.key_down(keycode);
        }
    }

    fn key_up_event(
        &mut self,
        _ctx: &mut ggez::Context,
        keycode: KeyCode,
        _keymods: KeyMods,
    ) {
        self.input.key_up(keycode);
    }

    fn gamepad_button_down_event(
        &mut self,
        _ctx: &mut ggez::Context,
        btn: Button,
        id: GamepadId
    ) {
        self.input.button_down(btn, id);
    }

    fn gamepad_axis_event(
        &mut self,
        _ctx: &mut ggez::Context,
        axis: Axis,
        value: f32,
        id: GamepadId
    ) {
        self.input.axis(axis, value, id);
    }

    fn update(&mut self, cx: &mut ggez::Context) -> ggez::GameResult {
        let mut cx = Context::new(cx, &self.script_engine, &mut self.input);
        self.battle.update(&mut cx).unwrap();

        Ok(())
    }

    fn draw(&mut self, cx: &mut ggez::Context) -> ggez::GameResult {
        graphics::clear(cx, [0.0, 0.0, 0.0, 1.0].into());

        {
            let mut cx = Context::new(cx, &self.script_engine, &mut self.input);
            self.battle.draw(&mut cx).unwrap();
        }

        graphics::present(cx)
    }
}

/// A game context.
pub struct Context<'a> {
    ggez: &'a mut ggez::Context,
    script_engine: &'a Engine,
    input: &'a mut Input,
}

impl<'a> Context<'a> {
    pub fn new(
        ggez: &'a mut ggez::Context,
        script_engine: &'a Engine,
        input: &'a mut Input,
    ) -> Context<'a> {
        Context { ggez, script_engine, input }
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

