//! Input data and structs.

use std::fmt::{self, Debug, Formatter};
use std::sync::{Arc, RwLock};
use std::hash::{Hash, Hasher};
use std::ops::{BitOr, BitOrAssign, BitAnd, BitAndAssign, Not};

use bytemuck::{Pod, Zeroable};

/// A buffer for inputs. Not to be confused with an
/// ["input buffer"](https://supersmashbros.fandom.com/wiki/Input_Buffering).
///
/// This is a very sepcial input data structure that uses a shared buffer to
/// efficiently use memory. Each input set corresponds to one frame of gameplay.
///
/// This struct also provides utility functions for reading special inputs,
/// directions, and reversal buffering.
#[derive(Clone, Default)]
pub struct Buffer(Arc<RwLock<Vec<Inputs>>>);

impl Buffer {
    /// Creates a new `Buffer`.
    pub fn new() -> Buffer {
        Buffer::default()
    }

    /// Returns how many inputs are in the buffer.
    pub fn len(&self) -> usize {
        self.0.read().unwrap().len()
    }

    /// Truncates the inputs to `len` frames.
    pub fn truncate(&self, len: usize) {
        self.0.write().unwrap().truncate(len)
    }

    /// Pushes a new set of inputs to the buffer.
    pub fn push(&self, input: Inputs) {
        self.0.write().unwrap().push(input)
    }

    /// The direction being held on the last frame.
    pub fn direction(&self) -> Direction {
        self.last().direction
    }

    /// The buttons being held on the last frame.
    pub fn buttons(&self) -> Buttons {
        self.last().buttons
    }

    /// The inputs being held on the last frame.
    pub fn last(&self) -> Inputs {
        *self.0.read().unwrap().last().unwrap()
    }
}

impl Debug for Buffer {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("Buffer(_)")
    }
}

impl Hash for Buffer {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        // the first input is the only relevant input when hashing
        self.0.read().unwrap().last().hash(state)
    }
}

/// A single frame of inputs.
#[derive(Clone, Copy, Default, PartialEq, Pod, Eq, Hash, Zeroable)]
#[repr(C)]
pub struct Inputs {
    /// The direction.
    pub direction: Direction,
    /// The buttons down.
    pub buttons: Buttons,
}

impl Debug for Inputs {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_tuple("Inputs")
            .field(&self.direction)
            .field(&self.buttons)
            .finish()
    }
}

/// Directional inputs.
///
/// Internally represented by [numpad notation][1].
///
/// [1]: http://www.dustloop.com/wiki/index.php/Notation
#[derive(Clone, Copy, PartialEq, Pod, Eq, Hash, Zeroable)]
#[repr(transparent)]
pub struct Direction(u8);

impl Direction {
    /// The neutral direction.
    ///
    /// This is returned in [`Direction::default()`].
    pub const D5: Direction = Direction(5);
    /// The down-left direction.
    pub const D1: Direction = Direction(1);
    /// The down direction.
    pub const D2: Direction = Direction(2);
    /// The down-right direction.
    pub const D3: Direction = Direction(3);
    /// The left direction.
    pub const D4: Direction = Direction(4);
    /// The right direction.
    pub const D6: Direction = Direction(6);
    /// The up-left direction.
    pub const D7: Direction = Direction(7);
    /// The up direction.
    pub const D8: Direction = Direction(8);
    /// The up-right direction.
    pub const D9: Direction = Direction(9);

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

impl Debug for Direction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Direction::D{}", self.0)
    }
}

impl Default for Direction {
    fn default() -> Direction {
        Direction::D5
    }
}

/// Button inputs.
#[derive(Clone, Copy, PartialEq, Pod, Eq, Hash, Zeroable)]
#[repr(transparent)]
pub struct Buttons(u8);

impl Buttons {
    /// The `P` (punch) input.
    pub const P: Buttons = Buttons(0b0001);
    /// The `K` (kick) input.
    pub const K: Buttons = Buttons(0b0010);
    /// The `S` (slash) input.
    pub const S: Buttons = Buttons(0b0100);
    /// The `H` (heavy slash) input.
    pub const H: Buttons = Buttons(0b1000);

    /// A list of buttons matched with string representations.
    pub const BUTTON_NAMES: &'static [(Buttons, &'static str)] = &[
        (Buttons::P, "P"),
        (Buttons::K, "K"),
        (Buttons::S, "S"),
        (Buttons::H, "H"),
    ];

    /// The empty set of buttons.
    pub const fn empty() -> Buttons {
        Buttons(0)
    }

    /// The complete set of buttons.
    pub const fn all() -> Buttons {
        Buttons(Buttons::P.0 | Buttons::K.0 | Buttons::S.0 | Buttons::H.0)
    }

    /// Checks if `self` is empty.
    pub const fn is_empty(self) -> bool {
        self.0 == Buttons::empty().0
    }

    /// Removes the buttons in `other` from `self`.
    pub fn remove(&mut self, other: Buttons) {
        *self &= !other;
    }

    /// Inserts the buttons in `other` to `self`.
    pub fn insert(&mut self, other: Buttons) {
        *self |= other;
    }

    /// Checks if all buttons in `other` are in `self`.
    pub const fn contains(self, other: Buttons) -> bool {
        self.0 & other.0 == other.0
    }
}

impl Debug for Buttons {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.is_empty() {
            f.write_str("(empty)")
        } else {
            let mut first = true;
            for &(button, name) in Buttons::BUTTON_NAMES {
                if self.contains(button) {
                    if first {
                        first = false;
                    } else {
                        f.write_str(" | ")?;
                    }

                    write!(f, "Buttons::{}", name)?;
                }
            }

            Ok(())
        }
    }
}

impl Default for Buttons {
    fn default() -> Buttons {
        Buttons::empty()
    }
}

impl BitOr for Buttons {
    type Output = Buttons;

    fn bitor(self, rhs: Buttons) -> Buttons {
        Buttons(self.0 | rhs.0)
    }
}

impl BitOrAssign for Buttons {
    fn bitor_assign(&mut self, rhs: Buttons) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for Buttons {
    type Output = Buttons;

    fn bitand(self, rhs: Buttons) -> Buttons {
        Buttons(self.0 & rhs.0)
    }
}

impl BitAndAssign for Buttons {
    fn bitand_assign(&mut self, rhs: Buttons) {
        self.0 &= rhs.0;
    }
}

impl Not for Buttons {
    type Output = Buttons;
    
    fn not(self) -> Buttons {
        Buttons(!self.0 & Buttons::all().0)
    }
}

