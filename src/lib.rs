//! # `bftd`

#[macro_use]
extern crate anyhow;

pub mod battle;
pub mod fsm;
pub mod rect;

use std::sync::Arc;

use glam::{Mat4, Mat3, Vec2};

use ggez::Context;
use ggez::event::EventHandler;
use ggez::graphics::{self, Image};

use battle::Battle;
use fsm::{Fsm, State, Frame, Sprite};

use anyhow::Error;

/// The main game state.
pub struct Game {
    battle: Battle,
}

impl Game {
    /// Creates the main game state.
    pub fn new(cx: &mut Context) -> Result<Game, Error> {
        let gdfsm = granddad_fsm(cx)?;

        /*
        let rect = ggez::graphics::Rect {
            x: 0.0,
            y: 480.0,
            w: 640.0,
            h: -480.0,
        };
        graphics::set_screen_coordinates(cx, rect).unwrap();
        */

        Ok(Game {
            battle: Battle::new(gdfsm.clone(), gdfsm),
        })
    }
}

impl EventHandler for Game {
    fn update(&mut self, cx: &mut Context) -> ggez::GameResult {
        Ok(())
    }

    fn draw(&mut self, cx: &mut Context) -> ggez::GameResult {
        graphics::clear(cx, [0.0, 0.0, 0.0, 1.0].into());

        self.battle.draw(cx).unwrap();

        graphics::present(cx)
    }
}

fn granddad_fsm(cx: &mut Context) -> Result<Fsm, Error> {
    // load idle texture
    let texture = include_bytes!("../granddad.png");
    let image = Image::from_bytes(cx, texture)?;

    let idle = State {
        name: Arc::from("idle"),
        frames: vec![
            Frame {
                sprite: Some(Sprite::new(image)),
            }
        ],
    };

    Ok(Fsm::new([idle]))
}

