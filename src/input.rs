//! Input scanning and reading.

use std::fmt::{self, Debug, Formatter};

/// A view into a set of inputs for each frame.
///
/// Provides utility functions for reading special inputs, directions, among
/// other things.
pub struct View<T: AsRef<[Inputs]>> {
    inputs: T
}

impl<T: AsRef<[Inputs]>> View<T> {
    /// Creates a new `View`.
    ///
    /// # Panics
    /// Panics if `inputs` is empty.
    pub fn new(inputs: T) -> View<T> {
        assert!(inputs.as_ref().len() > 0);

        View {
            inputs,
        }
    }

    /// The inputs inside the view.
    pub fn inputs(&self) -> &[Inputs] {
        self.inputs.as_ref()
    }

    /// The direction being held on the last frame.
    pub fn direction(&self) -> Direction {
        self.inputs().last().unwrap().direction
    }
}

/// A single frame of inputs.
#[derive(Clone, Copy, Default)]
pub struct Inputs {
    /// The direction.
    pub direction: Direction,
}

impl Debug for Inputs {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_tuple("Inputs")
            .field(&self.direction)
            .finish()
    }
}

/// Directional inputs.
///
/// Internally represented by [numpad notation][1].
///
/// [1]: http://www.dustloop.com/wiki/index.php/Notation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Direction {
    /// The neutral direction.
    ///
    /// This is returned in [`Direction::default()`].
    D5 = 5,
    /// The down-left direction.
    D1 = 1,
    /// The down direction.
    D2 = 2,
    /// The down-right direction.
    D3 = 3,
    /// The left direction.
    D4 = 4,
    /// The right direction.
    D6 = 6,
    /// The up-left direction.
    D7 = 7,
    /// The up direction.
    D8 = 8,
    /// The up-right direction.
    D9 = 9,
}

impl Direction {
    /// Flips the direction horizontally.
    ///
    /// # Examples
    /// ```
    /// # use bftd::input::Direction;
    /// let input = Direction::D3; // a launch input
    /// 
    /// assert_ne!(input.flip(), Direction::D3); // no longer launches!
    ///                                          // your input has been ruined!
    /// assert_eq!(input.flip(), Direction::D1);
    /// ```
    pub fn flip(self) -> Direction {
        match self {
            // flip down-left/right
            Direction::D1 => Direction::D3,
            Direction::D3 => Direction::D1,
            // flip left/right
            Direction::D4 => Direction::D6,
            Direction::D6 => Direction::D4,
            // flip up-left/right
            Direction::D7 => Direction::D9,
            Direction::D9 => Direction::D7,
            // no flipping has to be done for D5, D2 and D8
            d => d,
        }
    }
}

impl Default for Direction {
    fn default() -> Direction {
        Direction::D5
    }
}

