//! Battles!
//!
//! Not to sugarcoat it, but the entire game is basically implemented here, save
//! for the character select and menus.
//!
//! Someday, there may be an interactive online lobby system, but this is a
//! practical joke, so I highly doubt this'll get that far.
//!
//! Unless...?
//!
//! # Lifecycle
//! The lifecycle of each frame comes in a couple stages:
//! * **Inputs**  
//!   Inputs are sampled by a player's sampler and pushed to the front of that
//!   player's input queue.
//! * **Flip**
//!   Character players are flipped if they need to be.
//! * **Update**  
//!   The players' and projectiles' individual states are updated parallel to
//!   each other. If there is a state change, this stage is repeated for that
//!   entity.
//! * **Collide**  
//!   The game will attempt to process hitboxes, hurtboxes and collision boxes
//!   and update their states accordingly.

pub mod script;

use crate::fsm::{Key, Fsm};
use crate::input::{self, Inputs};
use crate::Context;

use script::Scope;

use glam::f32::{Affine2, Vec2};

use anyhow::Error;

/// How many frames of logic are elapsed in a single second.
pub const FRAMES_PER_SECOND: u32 = 60;

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
pub struct Arena {
    p1: Player,
    p2: Player,
}

impl Arena {
    /// Creates a battle with p1 and p2 initialized with [`Fsm`]s `p1` and `p2`.
    ///
    /// The initial state is always `"idle"`.
    pub fn new(p1: Fsm, p2: Fsm) -> Arena {
        Arena {
            p1: Player::new(p1, State::initial_p1()),
            p2: Player::new(p2, State::initial_p2()),
        }
    }

    /// Processes the next frame of gameplay using the inputs provided for each
    /// player.
    pub fn update(&mut self, cx: &mut Context, p1: Inputs, p2: Inputs) -> Result<(), Error> {
        // do flip post-processing after update
        if self.p1.pos().x < self.p2.pos().x {
            self.p1.state_mut().flipped = false;
            self.p2.state_mut().flipped = true;
        } else {
            self.p1.state_mut().flipped = true;
            self.p2.state_mut().flipped = false;
        }

        // first, update each player's individual state
        self.p1.update(cx, p1)?;
        self.p2.update(cx, p2)?;

        Ok(())
    }

    /// Draws the battle to a graphics context.
    pub fn draw(&self, cx: &mut Context) -> Result<(), Error> {
        self.p2.draw(cx)?;
        self.p1.draw(cx)
    }
}

/// One of two players in a battle.
pub struct Player {
    state: State,
    fsm: Fsm,
    scope: Scope<'static>,
    inputs: input::Buffer,
}

impl Player {
    /// Creates a new `Player`.
    ///
    /// The player will start with the `initial_state` passed to it.
    pub fn new(
        fsm: Fsm,
        initial_state: State,
    ) -> Player {
        Player {
            state: initial_state,
            fsm,
            scope: Scope::new(),
            inputs: Default::default(),
        }
    }
    
    /// The player's state.
    pub fn state(&self) -> &State {
        &self.state
    }

    /// A mutable reference to the player's state.
    pub fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }

    /// The position of the player.
    pub fn pos(&self) -> Vec2 {
        self.state.pos
    }

    /// Updates the player's state in respect to the inputs given.
    pub fn update(
        &mut self,
        cx: &mut Context,
        inputs: Inputs,
    ) -> Result<(), Error> {
        // add the input; gaurantees we never have a zero-size input
        self.inputs.push(inputs);
        self.scope.push("inputs", self.inputs.clone());

        let state = &self.fsm.get(&self.state.key)
            .ok_or_else(|| anyhow!("player in an invalid state"))?;
        while let Some(script) = &state.script {
            // run the script and update the character's state
            self.scope.push("state", self.state.clone());
            cx.script_engine
                .call_fn(&mut self.scope, script, "onupdate", ())
                .map_err(|e| anyhow!("{}", e))?;

            // see how the script updated the state
            let state = self.scope
                .get_value::<State>("state")
                .expect("state should exist");

            if state != self.state {
                // TODO: do state processing

                // update the state
                let state_switch = state.key != self.state.key;
                self.state = state;

                if !state_switch {
                    break;
                }
            } else {
                // break if there was no change
                break;
            }
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

            if self.state.flipped {
                transform = transform * Affine2::from_scale(Vec2::new(-1.0, 1.0));
            }

            sprite.draw(cx, transform)?;
        }
        
        Ok(())
    }
}

/// An entity's state.
///
/// This is what will be saved in the frame snapshot when rollback is
/// implemented. It should be cheaply cloneable and exposable to the scripting
/// framework.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    /// The position of the entity in the [`Arena`].
    pub pos: Vec2,
    /// If the entity is flipped. Entities normally face right, so if the entity
    /// is `flipped`, they would be facing left.
    pub flipped: bool,

    /// The key of the state of the entity.
    pub key: Key,
    /// The frame of the state of the entity.
    pub frame: usize,
}

impl State {
    /// Creates a new, initial state for player one.
    fn initial_p1() -> State {
        State {
            pos: Vec2::new(-500., 0.),
            flipped: false,
            key: Key::from("idle"),
            frame: 0,
        }
    }

    /// Creates a new, initial state for player two.
    fn initial_p2() -> State {
        State {
            pos: Vec2::new(500., 0.),
            flipped: true,
            key: Key::from("idle"),
            frame: 0,
        }
    }
}

