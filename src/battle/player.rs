//! Provies the [`Player`] struct.

use crate::fsm::{Fsm, Key};

use glam::f32::{Affine2, Vec2};

use anyhow::Error;

/// One of two players in a battle.
///
/// Handles the flipping of characters for you.
pub struct Player {
    pos: Vec2,
    facing_right: bool,
    state: State,
    
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
            pos,
            facing_right,
            state: State {
                key: initial_state,
                frame: 0,
            },

            fsm,
        }
    }

    /// The position of the player.
    pub fn pos(&self) -> Vec2 {
        self.pos
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
            let mut transform = Affine2::from_translation(self.pos);

            if self.facing_right {
                transform = transform * Affine2::from_scale(Vec2::new(-1.0, 1.0));
            }

            sprite.draw(cx, transform)?;
        }
        
        Ok(())
    }
}

struct State {
    key: Key,
    frame: usize,
}

