//! Input sampling and management.

use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};

use ggez::input::keyboard::KeyCode;
use ggez::input::gamepad::GamepadId;
use ggez::event::{Axis, Button};

use crate::input::{Direction, Inputs, Buttons};

/// The input controller.
///
/// Processes input events from the OS's push event stream and turns it into a
/// pullable stream of platform-independent inputs.
#[derive(Default)]
pub struct Input {
    samplers: Vec<Option<Sampler>>,
}

/// A handle to a single input device.
#[derive(Clone, Copy)]
pub struct Handle(usize);

impl Handle {
    // handle for testing
    #[doc(hidden)]
    pub fn new(u: usize) -> Handle {
        Handle(u)
    }
}

impl Input {
    /// Creates a new input controller from raw input systems.
    pub fn new(cx: &mut ggez::Context) -> Input {
        let mut samplers = Vec::new();

        for (id, _) in cx.gamepad_context.gamepads() {
            samplers.push(Sampler::Gamepad(Gamepad::new(id, GamepadMapping::default())));
        }

        // add keyboard mappings
        samplers.push(Sampler::Keyboard(Keyboard::new(KeyboardMapping::default())));

        Input {
            samplers: samplers.into_iter().map(|s| Some(s)).collect(),
        }
    }

    /// Iterates over the input devices.
    pub fn iter(&self) -> impl Iterator<Item = Handle> {
        self.samplers
            .iter()
            .enumerate()
            .filter(|(_, s)| s.is_some())
            .map(|(i, _)| Handle(i))
            .collect::<Vec<Handle>>()
            .into_iter()
    }

    /// Samples a set of inputs.
    ///
    /// Returns `None` if the handle is now invalid.
    pub fn sample(&mut self, id: Handle) -> Option<Inputs> {
        self.samplers
            .get_mut(id.0)
            .map(|s| s.as_mut().map(|s| s.sample()))
            .flatten()
    }

    /// Processes a key down event.
    pub fn key_down(&mut self, keycode: KeyCode) {
        for s in self.samplers_mut() {
            match s {
                Sampler::Keyboard(k) => k.key_down(keycode),
                _ => (),
            }
        }
    }

    /// Processes a key up event.
    pub fn key_up(&mut self, keycode: KeyCode) {
        for s in self.samplers_mut() {
            match s {
                Sampler::Keyboard(k) => k.key_up(keycode),
                _ => (),
            }
        }
    }

    /// Processes a gamepad button event.
    pub fn button_down(&mut self, btn: Button, id: GamepadId) {
        for s in self.samplers_mut() {
            match s {
                Sampler::Gamepad(g) if g.id == id => g.button_down(btn),
                _ => (),
            }
        }
    }

    /// Processes a gamepad axis event.
    pub fn axis(&mut self, axis: Axis, value: f32, id: GamepadId) {
        for s in self.samplers_mut() {
            match s {
                Sampler::Gamepad(g) if g.id == id => g.axis(axis, value),
                _ => (),
            }
        }
    }

    fn samplers_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut Sampler> {
        self.samplers.iter_mut().filter_map(|s| s.as_mut())
    }
}

impl Debug for Handle {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

/// A sampler.
#[derive(Debug)]
pub enum Sampler {
    Keyboard(Keyboard),
    Gamepad(Gamepad),
}

impl Sampler {
    /// Samples a set of inputs.
    pub fn sample(&mut self) -> Inputs {
        match self {
            Sampler::Keyboard(k) => k.sample(),
            Sampler::Gamepad(g) => g.sample(),
        }
    }
}

/// The left direction.
const DIRECTION_LEFT: u8 = 0b1000;
/// The right direction.
const DIRECTION_RIGHT: u8 = 0b0100;
/// The up direction.
const DIRECTION_UP: u8 = 0b0001;
/// The down direction.
const DIRECTION_DOWN: u8 = 0b0010;

/// A keyboard sampler.
#[derive(Debug)]
pub struct Keyboard {
    direction: u8,
    buttons: Buttons,
    mapping: KeyboardMapping,
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
    pub fn new(mapping: KeyboardMapping) -> Keyboard {
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
    }

    /// Samples the last frame of inputs.
    pub fn sample(&mut self) -> Inputs {
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

        let inputs = Inputs {
            direction,
            buttons: self.buttons,
        };

        self.buttons = Buttons::empty();
        inputs
    }
}

/// A mapping for keyboard inputs.
#[derive(Debug)]
pub struct KeyboardMapping {
    direction_map: HashMap<KeyCode, u8>,
    button_map: HashMap<KeyCode, Buttons>,
}

impl Default for KeyboardMapping {
    fn default() -> KeyboardMapping {
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

        KeyboardMapping { direction_map, button_map }
    }
}

/// A gamepad sampler.
#[derive(Debug)]
pub struct Gamepad {
    id: GamepadId,
    axis_x: f32,
    axis_y: f32,
    buttons: Buttons,
    mapping: GamepadMapping,
}

impl Gamepad {
    /// Creates a new `Gamepad` sampler.
    pub fn new(id: GamepadId, mapping: GamepadMapping) -> Gamepad {
        Gamepad {
            id,
            axis_x: 0.,
            axis_y: 0.,
            buttons: Buttons::default(),
            mapping,
        }
    }

    /// Processes a gamepad button event.
    pub fn button_down(&mut self, btn: Button) {
        if let Some(&buttons) = self.mapping.button_map.get(&btn) {
            self.buttons.insert(buttons);
        }
    }

    /// Processes a gamepad axis event.
    pub fn axis(&mut self, axis: Axis, value: f32) {
        match axis {
            Axis::LeftStickX => self.axis_x = value,
            Axis::LeftStickY => self.axis_y = value,
            _ => (),
        }
    }

    /// Samples the last frame of inputs.
    pub fn sample(&mut self) -> Inputs {
        let angle = self.axis_y.atan2(self.axis_x) * (180. / std::f32::consts::PI);
        let mag = self.axis_x * self.axis_x + self.axis_y * self.axis_y;
        let deadzone2 = self.mapping.deadzone * self.mapping.deadzone;

        let direction = if mag < deadzone2 {
            Direction::D5
        } else {
            match angle {
                a if a > -157.5  && a <= -112.5 => Direction::D1,
                a if a > -112.5  && a <= -67.5  => Direction::D2,
                a if a > -67.5   && a <= -22.5  => Direction::D3,
                a if a > -22.5   && a <= 22.5   => Direction::D6,
                a if a > 22.5    && a <= 67.5   => Direction::D9,
                a if a > 67.5    && a <= 112.5  => Direction::D8,
                a if a > 112.5   && a <= 157.5  => Direction::D7,
                a if a > 157.5   && a <= 180.0  => Direction::D4,
                a if a >= -180.0 && a <= -157.5 => Direction::D4,
                _ => unreachable!(),
            }
        };

        let inputs = Inputs {
            direction,
            buttons: self.buttons,
        };

        self.buttons = Buttons::empty();
        inputs
    }
}

/// Gamepad mapping.
#[derive(Debug)]
pub struct GamepadMapping {
    button_map: HashMap<Button, Buttons>,
    deadzone: f32,
}

impl Default for GamepadMapping {
    fn default() -> GamepadMapping {
        let mut button_map = HashMap::new();

        button_map.insert(Button::South, Buttons::K);
        button_map.insert(Button::West, Buttons::P);
        button_map.insert(Button::North, Buttons::S);
        button_map.insert(Button::East, Buttons::H);

        GamepadMapping { button_map, deadzone: 0.1 }
    }
}

