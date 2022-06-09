//! Finite-state machines implemented by [`Fsm`].

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

use crate::battle::script::AST;
use crate::render::Sprite;

/// A cheaply-cloneable key for a finite-state machine entry.
pub type Key = Arc<str>;

/// Some finite-state machine code.
///
/// An entity controlled by an [`Fsm`] has some common properties:
/// * Has an origin on the stage.
/// * Creates hitboxes and hurtboxes.
/// * May have a sprite to be rendered to the screen.
///
/// `Fsm`s are **not** meant to be mutable. As such, an `Fsm` will only allow
/// immutable access to it's contents. However, it also means that `Fsm` is very
/// cheaply cloneable.
///
/// # Scripting
/// Using [`rhai`], different states can be given a state script, which holds
/// callbacks that are called upon certain events.
/// * `onenter` is called when the `Fsm` enters the state.
/// * `onupdate` is called when a frame advances occurs while the state is
/// active. This is called every frame the state is active, including the frame
/// that `onenter` is called.
/// * `onexit` is called when the `Fsm` leaves the state, after `onupdate` is
/// called.
///
/// In an event context, certain functions are exposed to get the current state,
/// update the current state or swap the state with a new one entirely. Other
/// functions are provided to read input, manage super freezes, or simply moving
/// the character. See the `assets/core/` folder for shared scripts for
/// character moveset.
#[derive(Clone, Debug)]
pub struct Fsm {
    states: Arc<HashMap<Key, State>>,
}

impl Fsm {
    /// Creates an `Fsm` from a list of states.
    pub fn new(states: impl IntoIterator<Item = State>) -> Fsm {
        let states = states
            .into_iter()
            .map(|state| {
                let key = state.name.clone();
                (key, state)
            })
            .collect();

        Fsm {
            states: Arc::new(states),
        }
    }
}

impl Deref for Fsm {
    type Target = HashMap<Key, State>;

    fn deref(&self) -> &Self::Target {
        self.states.deref()
    }
}

/// A single state in a [`Fsm`].
#[derive(Clone, Debug)]
pub struct State {
    /// The name of the state.
    pub name: Key,
    /// The list of frames in the state.
    pub frames: Vec<Frame>,
    /// The script of the state, if there is one.
    pub script: Option<AST>,
}

impl State {
    /// The amount of frames in the state.
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    /// Selects a specific frame.
    ///
    /// Does not panic if the index is out of bounds.
    pub fn frame(&self, n: usize) -> Option<&Frame> {
        if n < self.len() {
            Some(&self.frames[n])
        } else {
            None
        }
    }
}

/// A single frame in a [`State`].
#[derive(Clone, Debug)]
pub struct Frame {
    /// The sprite to display for this frame.
    pub sprite: Option<Sprite>,
}
