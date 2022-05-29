//! Keyboard sampling.

use std::collections::HashMap;

use ggez::input::keyboard::KeyCode;

use crate::input::{Direction, Inputs, Buttons};

/// The left direction.
pub const DIRECTION_LEFT: u8 = 0b1000;
/// The right direction.
pub const DIRECTION_RIGHT: u8 = 0b0100;
/// The up direction.
pub const DIRECTION_UP: u8 = 0b0001;
/// The down direction.
pub const DIRECTION_DOWN: u8 = 0b0010;

/// A keyboard sampler.
pub struct Keyboard {
    direction: u8,
    buttons: Buttons,
    mapping: Mapping,
}

// TODO: this implementation is extremely quick and dirty, find a more elegant
// solution to this!
//
// to be honest, we may just want to use an adapter to adapt keyboard inputs to
// gamepad inputs, so we really only have to implement the gamepad input
// handler, but it may not work as well.
// TODO: also fix input tunneling
impl Keyboard {
    /// Creates a new `Keyboard` sampler.
    pub fn new(mapping: Mapping) -> Keyboard {
        Keyboard {
            mapping,
            direction: 0,
            buttons: Buttons::empty(),
        }
    }

    /// Processes a key down event.
    pub fn key_down(&mut self, key: KeyCode) {
        if let Some(&direction) = self.mapping.direction_map.get(&key) {
            self.direction |= direction;
        }

        if let Some(&buttons) = self.mapping.button_map.get(&key) {
            self.buttons.insert(buttons);
        }
    }

    /// Processes a key up event.
    pub fn key_up(&mut self, key: KeyCode) {
        if let Some(&direction) = self.mapping.direction_map.get(&key) {
            self.direction &= !direction;
        }

        if let Some(&buttons) = self.mapping.button_map.get(&key) {
            self.buttons.remove(buttons);
        }
    }

    /// Samples the last frame of inputs.
    pub fn sample(&self) -> Inputs {
        let x_axis = ((self.direction >> 2) & 0b11) % 0b11;
        let y_axis = (self.direction & 0b11) % 0b11;

        let direction = match (x_axis, y_axis) {
            (0b10, 0b10) => Direction::D1,
            (0b00, 0b10) => Direction::D2,
            (0b01, 0b10) => Direction::D3,
            (0b10, 0b00) => Direction::D4,
            (0b01, 0b00) => Direction::D6,
            (0b10, 0b01) => Direction::D7,
            (0b00, 0b01) => Direction::D8,
            (0b01, 0b01) => Direction::D9,
            _ => Direction::D5,
        };

        Inputs {
            direction,
            buttons: self.buttons,
        }
    }
}

/// A mapping for keyboard inputs.
pub struct Mapping {
    direction_map: HashMap<KeyCode, u8>,
    button_map: HashMap<KeyCode, Buttons>,
}

impl Default for Mapping {
    fn default() -> Mapping {
        let mut direction_map = HashMap::new();

        direction_map.insert(KeyCode::W, DIRECTION_UP);
        direction_map.insert(KeyCode::S, DIRECTION_DOWN);
        direction_map.insert(KeyCode::A, DIRECTION_LEFT);
        direction_map.insert(KeyCode::D, DIRECTION_RIGHT);

        let mut button_map = HashMap::new();

        button_map.insert(KeyCode::U, Buttons::P);
        button_map.insert(KeyCode::I, Buttons::K);
        button_map.insert(KeyCode::O, Buttons::S);
        button_map.insert(KeyCode::P, Buttons::H);

        Mapping { direction_map, button_map }
    }
}

