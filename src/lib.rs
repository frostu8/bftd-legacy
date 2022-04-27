//! # `bftd`

#[macro_use]
extern crate anyhow;

pub mod battle;
pub mod fsm;
pub mod input;
pub mod rect;

use std::sync::Arc;

use glam::Vec2;

use ggez::{Context, timer};
use ggez::event::EventHandler;
use ggez::input::keyboard::{self, KeyCode, KeyMods};
use ggez::graphics::{self, Image};

use battle::{Battle, FRAMES_PER_SECOND};
use input::{Direction, Inputs};
use fsm::{Fsm, State, Frame, Sprite};

use anyhow::Error;

/// The main game state.
pub struct Game {
    battle: Battle,
    current_inputs: Inputs,
}

impl Game {
    /// Creates the main game state.
    pub fn new(cx: &mut Context) -> Result<Game, Error> {
        const ELEVATION: f32 = 100.0;

        let gdfsm = granddad_fsm(cx)?;

        let rect = ggez::graphics::Rect {
            x: -960.0,
            y: -1080.0 + ELEVATION,
            w: 1920.0,
            h: 1080.0,
        };
        graphics::set_screen_coordinates(cx, rect).unwrap();

        Ok(Game {
            battle: Battle::new(gdfsm.clone(), gdfsm),
            current_inputs: Default::default(),
        })
    }
}

impl EventHandler for Game {
    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool
    ) {
        if keycode == KeyCode::Escape {
            ggez::event::quit(ctx);
            return;
        }

        // TODO: inputs are only sampled at the start of the frame;
        // this enables input tunneling that is very bad and needs to be fixed
    }

    fn update(&mut self, cx: &mut Context) -> ggez::GameResult {
        while timer::check_update_time(cx, FRAMES_PER_SECOND) {
            // do final pass of inputs
            self.current_inputs = poll_inputs(cx);

            // update
            self.battle.update(self.current_inputs, Inputs::default()).unwrap();
            self.current_inputs = Inputs::default();
        }

        Ok(())
    }

    fn draw(&mut self, cx: &mut Context) -> ggez::GameResult {
        graphics::clear(cx, [0.0, 0.0, 0.0, 1.0].into());

        self.battle.draw(cx).unwrap();

        graphics::present(cx)
    }
}

fn poll_inputs(cx: &mut Context) -> Inputs {
    if keyboard::is_key_pressed(cx, KeyCode::D) {
        Inputs {
            direction: Direction::D6,
        }
    } else {
        Inputs::default()
    }
}

fn granddad_fsm(cx: &mut Context) -> Result<Fsm, Error> {
    // load idle texture
    let texture = include_bytes!("../granddad.png");
    let image = Image::from_bytes(cx, texture)?;

    let mut idle_sprite = Sprite::new(image);
    idle_sprite.transform = glam::Affine2::from_scale(Vec2::new(0.625, 0.625));

    let idle = State {
        name: Arc::from("idle"),
        frames: vec![
            Frame {
                sprite: Some(idle_sprite),
            }
        ],
    };

    Ok(Fsm::new([idle]))
}

