//! Battles!
//!
//! Not to sugarcoat it, but the entire game is basically implemented here, save
//! for the character select and menus.
//!
//! Someday, there may be an interactive online lobby system, but this is a
//! practical joke, so I highly doubt this'll get that far.
//!
//! Unless...?

mod player;

pub use player::Player;
use crate::fsm::Fsm;

use std::sync::Arc;

use glam::f32::Vec2;

use anyhow::Error;

/// The size of each stage in the game.
///
/// The origin of the stage is `0`. In the case of `10,000`, the stage would
/// extend `5,000` units to the left and `5,000` units to the right.
pub const STAGE_SIZE: f32 = 10_000.0;

/// The maximum horizontal distance two players can be away from each other.
pub const MAX_HORIZONTAL_DISTANCE: f32 = 3_000.0;

/// A battle.
///
/// Handles the updating and rendering of the battle to the screen. Does
/// **not** handle background shaders; you can go crazy with that.
pub struct Battle {
    p1: Player,
    p2: Player,
}

impl Battle {
    /// Creates a battle with p1 and p2 initialized with [`Fsm`]s `p1` and `p2`.
    ///
    /// The initial state is always `"idle"`.
    pub fn new(p1: Fsm, p2: Fsm) -> Battle {
        Battle {
            p1: Player::new(p1, Vec2::new(-500., 0.), Arc::from("idle"), false),
            p2: Player::new(p2, Vec2::new(500., 0.), Arc::from("idle"), true),
        }
    }

    /// Draws the battle to a graphics context.
    pub fn draw(&self, cx: &mut ggez::Context) -> Result<(), Error> {
        self.p2.draw(cx)?;
        self.p1.draw(cx)
    }
}

