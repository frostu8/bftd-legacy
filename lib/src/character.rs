//! Character modules.

use crate::Rect;
use serde::{Serialize, Deserialize};
use glam::f32::Affine2;

/// A character definition.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Character {
    /// The internal id of the character. This does not need to be the same as
    /// the filename, but it should be.
    pub id: String,
    /// The states of the character.
    pub states: Vec<State>,
}

/// A state.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct State {
    /// The name of the state.
    pub name: String,
    /// The list of frames in the state.
    pub frames: Vec<Frame>,
    /// A path to the script of the state.
    pub script: Option<String>,
}

/// A single frame in a [`State`].
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Frame {
    /// The sprite to display for this frame.
    pub sprite: Option<Sprite>,
}

/// A [`Frame`]'s sprite.
///
/// The origin is the bottom-center of the sprite.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Sprite {
    /// A path to the texture of the state.
    pub texture: String,
    /// The source rectangle of the image.
    #[serde(default = "default_rect")]
    pub src: Rect,
    /// The transformations to be applied to the image, relative to the origin.
    #[serde(default)]
    pub transform: Affine2,
}

fn default_rect() -> Rect {
    Rect::new_wh(0., 0., 1., 1.)
}

