//! Provies the [`Player`] struct.

use crate::fsm::{Fsm, Key};
use crate::input::{Direction, Inputs, View};

use glam::f32::{Affine2, Vec2};

use anyhow::Error;

/// One of two players in a battle.
///
/// Handles the flipping of characters for you, among other things.
pub struct Player {
    state: State,
    inputs: Vec<Inputs>,
    fsm: Fsm,
}

impl Player {
    /// Creates a new `Player`.
    ///
    /// The player will start at the `initial_state` on frame `0`.
    pub fn new(
        fsm: Fsm,
        pos: Vec2,
        initial_state: Key,
        facing_right: bool,
    ) -> Player {
        Player {
            state: State {
                pos,
                facing_right,
                key: initial_state,
                frame: 0,
            },
            inputs: Vec::new(),
            fsm,
        }
    }

    /// The position of the player.
    pub fn pos(&self) -> Vec2 {
        self.state.pos
    }

    /// Updates the player's state in respect to the inputs given.
    pub fn update(&mut self, inputs: Inputs) -> Result<(), Error> {
        // add the input; gaurantees we never have a zero-size input
        self.inputs.push(inputs);
        let view = View::new(&self.inputs);

        if view.last() != Inputs::default() {
            println!("{:?}", view.last());
        }

        Ok(())
    }

    /// Draws the player to the screen.
    pub fn draw(&self, cx: &mut ggez::Context) -> Result<(), Error> {
        let sprite = &self.fsm
            .get(&self.state.key)
            .ok_or_else(|| anyhow!("player in an invalid state"))?
            .frame(self.state.frame)
            .ok_or_else(|| anyhow!("player in an invalid frame"))?
            .sprite;

        if let Some(sprite) = sprite {
            let mut transform = Affine2::from_translation(self.state.pos);

            if self.state.facing_right {
                transform = transform * Affine2::from_scale(Vec2::new(-1.0, 1.0));
            }

            sprite.draw(cx, transform)?;
        }
        
        Ok(())
    }
}

/// Player state, intended to be saved for uses in rollback.
struct State {
    pos: Vec2,
    facing_right: bool,

    key: Key,
    frame: usize,
}

