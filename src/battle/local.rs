//! Local battle manager.

use crate::input::{sampler::{self, Handle}, Buffer as InputBuffer};
use crate::render::Renderer;
use crate::Context;

use super::{Arena, FRAMES_PER_SECOND};

use anyhow::Error;

/// A local battle manager.
pub struct LocalBattle {
    p1: Player,
    p2: Player,
    arena: Arena,
}

struct Player {
    id: sampler::Handle,
    inputs: InputBuffer,
}

impl LocalBattle {
    /// Creates a new `LocalBattle` with input handles.
    pub fn new(arena: Arena, p1: Handle, p2: Handle) -> LocalBattle {
        LocalBattle {
            arena,
            p1: Player { id: p1, inputs: Default::default() },
            p2: Player { id: p2, inputs: Default::default() },
        }
    }

    /// Polls an update for the `LocalBattle`.
    ///
    /// Because all of the input processing is done locally, this will wait
    /// until each frame is done processing.
    pub fn update(&mut self, cx: &mut Context) -> Result<(), Error> {
        while cx.frame_limiter.should_update(FRAMES_PER_SECOND) {
            // sample from our players
            self.p1.inputs.push(cx.input.sample(self.p1.id).unwrap_or_default());
            self.p2.inputs.push(cx.input.sample(self.p2.id).unwrap_or_default());

            self.arena.update(
                &cx.script,
                &self.p1.inputs,
                &self.p2.inputs,
            )?;
        }

        Ok(())
    }
    
    /// Draws the battle to a graphics context.
    pub fn draw(&mut self, cx: &mut Renderer) -> Result<(), Error> {
        self.arena.draw(cx)
    }
}

