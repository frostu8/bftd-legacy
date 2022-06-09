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
//! * **Update**  
//!   The players' and projectiles' individual states are updated parallel to
//!   each other. If there is a state change, this stage is repeated for that
//!   entity.
//! * **Flip**
//!   Character players are flipped if they need to be.
//! * **Collide**  
//!   The game will attempt to process hitboxes, hurtboxes and collision boxes
//!   and update their states accordingly.

pub mod fsm;
mod local;
mod net;
pub mod script;

pub use local::LocalBattle;
pub use net::{NetBattle, NetPlayer};

use crate::input::Buffer as InputBuffer;
use crate::render::{Drawable, Renderer};
use fsm::{Fsm, Key};

use std::hash::{Hash, Hasher};

use script::{Engine, Scope};

use glam::f32::{Affine2, Vec2};

use anyhow::Error;

/// How many frames of logic are elapsed in a single second.
pub const FRAMES_PER_SECOND: u64 = 60;

/// The size of each stage in the game.
///
/// The origin of the stage is `0`. In the case of `10,000`, the stage would
/// extend `5,000` units to the left and `5,000` units to the right.
pub const STAGE_SIZE: f32 = 10_000.0;

/// The maximum horizontal distance two players can be away from each other.
pub const MAX_HORIZONTAL_DISTANCE: f32 = 3_000.0;

/// A headless arena.
///
/// This only handles the frame-by-frame logic of updating the match state, the
/// player's positions, health bars and super freezes. It also has a
/// convenience function for drawing the battle to the screen.
pub struct Arena {
    p1: Player,
    p2: Player,
}

impl Arena {
    /// Creates a battle with p1 and p2 initialized with [`Fsm`]s `p1` and `p2`.
    ///
    /// The initial state is always `"idle"`.
    pub fn new(engine: &Engine, p1: Fsm, p2: Fsm) -> Result<Arena, Error> {
        Ok(Arena {
            p1: Player::new(engine, p1, State::initial_p1())?,
            p2: Player::new(engine, p2, State::initial_p2())?,
        })
    }

    /// Processes the next frame of gameplay using the inputs provided for each
    /// player.
    pub fn update(
        &mut self,
        engine: &Engine,
        p1: &InputBuffer,
        p2: &InputBuffer,
    ) -> Result<(), Error> {
        // first, update each player's individual state
        self.p1.update(engine, p1)?;
        self.p2.update(engine, p2)?;

        // do flip post-processing after update
        if self.p1.pos().x < self.p2.pos().x {
            self.p1.state_mut().flipped = false;
            self.p2.state_mut().flipped = true;
        } else {
            self.p1.state_mut().flipped = true;
            self.p2.state_mut().flipped = false;
        }

        Ok(())
    }

    /// Draws the battle to a graphics context.
    pub fn draw(&self, cx: &mut Renderer) -> Result<(), Error> {
        let aspect_ratio = 1. / cx.aspect_ratio();

        let min = self.p1.state.pos.min(self.p2.state.pos) - Vec2::new(0.4, 0.);
        let max = self.p1.state.pos.max(self.p2.state.pos) + Vec2::new(0.4, 2.0);

        let center = (min + max) / 2.;

        let scale_x = aspect_ratio / (max.x - min.x);
        let scale_y = 1. / max.y - min.y;

        let scale = scale_x.min(scale_y).min(0.4);

        cx.set_transform(
            Affine2::from_scale(Vec2::new(scale, scale)) * Affine2::from_translation(-center),
        );

        self.p2.draw(cx)?;
        self.p1.draw(cx)
    }
}

/// One of two players in a battle.
pub struct Player {
    fsm: Fsm,
    state: State,
    scope: Scope<'static>,
}

impl Player {
    /// Creates a new `Player`.
    ///
    /// The player will start with the `initial_state` passed to it. The engine
    /// is required to be passed to run initial logic.
    pub fn new(engine: &Engine, fsm: Fsm, initial_state: State) -> Result<Player, Error> {
        let mut player = Player {
            fsm,
            state: initial_state,
            scope: Scope::new(),
        };

        // evaluate idle script
        eval(&player.state.key, &player.fsm, engine, &mut player.scope)?;

        Ok(player)
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
    pub fn update(&mut self, engine: &Engine, inputs: &InputBuffer) -> Result<(), Error> {
        self.scope.push("inputs", inputs.clone());

        let state = &self
            .fsm
            .get(&self.state.key)
            .ok_or_else(|| anyhow!("player in an invalid state"))?;

        while let Some(script) = &state.script {
            // run the script and update the character's state
            self.scope.push("state", self.state.clone());
            engine.call_fn_raw(&mut self.scope, script, true, true, "onupdate", None, [])?;

            // see how the script updated the state
            let state = self
                .scope
                .get_value::<State>("state")
                .ok_or_else(|| anyhow!("script replaced `state` variable"))?;

            let key_old = self.state.key.clone();

            if state != self.state {
                // TODO: do state processing

                // update the state
                self.state = state;
            }

            if self.state.key != key_old {
                eval(&self.state.key, &self.fsm, engine, &mut self.scope)?;
            } else {
                break;
            }
        }

        Ok(())
    }

    /// Draws the player to the screen.
    pub fn draw(&self, cx: &mut Renderer) -> Result<(), Error> {
        let sprite = &self
            .fsm
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

            transform = transform * Affine2::from_translation(Vec2::new(0., 0.5));

            let mut sprite = sprite.clone();
            sprite.set_transform(sprite.transform() * transform);
            sprite.draw(cx);
        }

        Ok(())
    }
}

fn eval(key: &str, fsm: &Fsm, engine: &Engine, scope: &mut Scope<'static>) -> Result<(), Error> {
    let state = fsm
        .get(key)
        .ok_or_else(|| anyhow!("player in an invalid state"))?;

    if let Some(script) = &state.script {
        engine
            .eval_ast_with_scope::<()>(scope, script)
            .map_err(From::from)
    } else {
        Ok(())
    }
}

/// An entity's state.
///
/// This should be cheaply cloneable as it will be exposed to the scripting
/// framework later.
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
            pos: Vec2::new(-1., 0.),
            flipped: false,
            key: Key::from("idle"),
            frame: 0,
        }
    }

    /// Creates a new, initial state for player two.
    fn initial_p2() -> State {
        State {
            pos: Vec2::new(1., 0.),
            flipped: true,
            key: Key::from("idle"),
            frame: 0,
        }
    }
}

impl Hash for State {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        // write vec
        hasher.write(&self.pos.x.to_ne_bytes());
        hasher.write(&self.pos.y.to_ne_bytes());

        self.flipped.hash(hasher);
        self.key.hash(hasher);
        self.frame.hash(hasher);
    }
}
