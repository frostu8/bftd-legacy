//! Finite-state machines implemented by [`Fsm`].

use bftd_lib::Rect;

use glam::f32::{Affine2, Mat4, Vec2};

use crate::assets::Asset;

use std::rc::Rc;
use std::ops::Deref;
use std::collections::HashMap;

use rhai::AST;

use anyhow::Error;

/// A cheaply-cloneable key for a finite-state machine entry.
pub type Key = Rc<str>;

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
#[derive(Clone)]
pub struct Fsm {
    states: Rc<HashMap<Key, State>>,
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
            states: Rc::new(states),
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
pub struct Frame {
    /// The sprite to display for this frame.
    pub sprite: Option<Sprite>,
}

/// A [`Frame`]'s sprite.
#[derive(Clone, Debug)]
pub struct Sprite {
    /// The in-GPU memory of the source texture.
    pub texture: Asset<ggez::graphics::Image>,
    /// The source rectangle of the image.
    pub src: Rect,
    /// The transformations to be applied to the image, relative to the origin.
    pub transform: Affine2,
}

impl Sprite {
    /// Creates a new sprite from a raw GPU texture.
    pub fn new(texture: ggez::graphics::Image) -> Sprite {
        Sprite {
            src: Rect::new_wh(0., 0., 1., 1.),
            texture: Asset::new(texture),
            transform: Affine2::IDENTITY,
        }
    }

    fn offset(&self) -> Affine2 {
        Affine2::from_translation(
            -Vec2::new(self.width() as f32 / 2., self.height() as f32)
        )
    }

    /// The width of the untransformed sprite.
    pub fn width(&self) -> f32 {
        self.src.width() * self.texture.width() as f32
    }

    /// The height of the untransformed sprite.
    pub fn height(&self) -> f32 {
        self.src.height() * self.texture.height() as f32
    }

    /// Draws the sprite to a drawing context.
    pub fn draw(&self, cx: &mut ggez::Context, origin: Affine2) -> Result<(), Error> {
        // get transform
        let transform = origin * self.transform * self.offset();

        let params = ggez::graphics::DrawParam {
            /*src: ggez::graphics::Rect {
                x: self.src.left(), y: self.src.top(),
                w: self.src.width(), h: self.src.height(),
            },*/
            trans: to_ggez_transform(transform),
            ..Default::default()
        };

        // draw sprite to screen
        ggez::graphics::draw(cx, self.texture.as_ref(), params)
            .map_err(Into::into)
    }
}

fn to_ggez_transform(affine: Affine2) -> ggez::graphics::Transform {
    let mat = Mat4::from_cols(
        (affine.matrix2.col(0), 0.0, 0.0).into(),
        (affine.matrix2.col(1), 0.0, 0.0).into(),
        (0.0, 0.0, 1.0, 0.0).into(),
        (affine.translation, 0.0, 1.0).into(),
    );

    ggez::graphics::Transform::Matrix(mat.into())
}

