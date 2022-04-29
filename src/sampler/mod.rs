//! Input sampling utilities.

pub mod keyboard;

use crate::input::Inputs;

/// An input sampler.
///
/// An input sampler is responsible for processing input events between frame
/// updates and putting them together into one, platform-independent sample of
/// inputs.
pub enum Sampler {
    Keyboard(keyboard::Keyboard),
}

impl Sampler {
    /// Samples the inputs on the current frame.
    pub fn sample(&self) -> Inputs {
        match self {
            Sampler::Keyboard(k) => k.sample(),
        }
    }
}

