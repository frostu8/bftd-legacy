//! Input sampling and management.

use winit::event::ScanCode;

use gilrs::{
    ev::{Axis, Button},
    EventType, GamepadId, Gilrs,
};

use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};

use crate::input::{Buttons, Direction, Inputs};

use uuid::Uuid;

use serde::{Deserialize, Serialize};

/// The global input sampler.
///
/// Processes inputs from the OS and converts them to useful, platform-
/// independent events. [`gilrs`] already does an amazing job at this, but we
/// need to consider the keyboard as an input device.
pub struct Sampler {
    gilrs: Gilrs,
    bindings: Bindings,
    devices: Vec<Option<Device>>,
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

impl Sampler {
    /// Creates a new input controller from raw input systems.
    pub fn new(mut bindings: Bindings) -> Sampler {
        let gilrs = Gilrs::new().unwrap();
        let mut devices = Vec::new();

        // add keyboards
        for bindings in bindings.keyboards.iter() {
            devices.push(Some(Device::Keyboard(Keyboard::new(bindings.clone()))));
        }

        // iterate over gamepads
        for (id, gamepad) in gilrs.gamepads() {
            let uuid = Uuid::from_bytes(gamepad.uuid());
            let bindings = bindings.get(&uuid).clone();
            devices.push(Some(Device::Gamepad(Gamepad::new(id, uuid, bindings))));
        }

        Sampler {
            gilrs,
            bindings,
            devices,
        }
    }

    /// Iterates over the input devices.
    pub fn iter(&self) -> impl Iterator<Item = Handle> {
        self.devices
            .iter()
            .enumerate()
            .filter(|(_, s)| s.is_some())
            .map(|(i, _)| Handle(i))
            .collect::<Vec<Handle>>()
            .into_iter()
    }

    /// Samples a set of inputs.
    ///
    /// Returns `None` if the handle is invalid, possibly from unplugging a
    /// controller.
    pub fn sample(&mut self, id: Handle) -> Option<Inputs> {
        self.devices
            .get_mut(id.0)
            .map(|s| s.as_mut().map(|s| s.sample()))
            .flatten()
    }

    /// Polls lower level input constructs.
    pub fn poll(&mut self) {
        while let Some(ev) = self.gilrs.next_event() {
            match ev.event {
                EventType::ButtonPressed(btn, _) => self.process_button_down(ev.id, btn),
                EventType::AxisChanged(axis, value, _) => self.process_axis(ev.id, axis, value),
                EventType::Connected => {
                    let gamepad = self.gilrs.gamepad(ev.id);
                    let uuid = Uuid::from_bytes(gamepad.uuid());
                    let bindings = self.bindings.get(&uuid);
                    let device = Device::Gamepad(Gamepad::new(ev.id, uuid, bindings));

                    // find spot to put device
                    for d in self.devices.iter_mut() {
                        if let None = d {
                            *d = Some(device);
                            return;
                        }
                    }

                    // add new device to end if no spot was found
                    self.devices.push(Some(device));
                }
                EventType::Disconnected => {
                    for device in self.devices.iter_mut() {
                        if let Some(Device::Gamepad(gamepad)) = device {
                            if gamepad.id == ev.id {
                                *device = None;
                            }
                        }
                    }
                }
                _ => (),
            }
        }
    }

    /// Processes a key down event.
    pub fn process_key_down(&mut self, keycode: ScanCode) {
        for k in self.keyboards_mut() {
            k.key_down(keycode);
        }
    }

    /// Processes a key up event.
    pub fn process_key_up(&mut self, keycode: ScanCode) {
        for k in self.keyboards_mut() {
            k.key_up(keycode);
        }
    }

    /// Processes a gamepad button event.
    ///
    /// You shouldn't need to call this yourself. Call [`Sampler::poll`]
    /// instead.
    pub fn process_button_down(&mut self, id: GamepadId, btn: Button) {
        for g in self.gamepads_mut().filter(|g| g.id == id) {
            g.button_down(btn);
        }
    }

    /// Processes a gamepad axis event.
    ///
    /// You shouldn't need to call this yourself. Call [`Sampler::poll`]
    /// instead.
    pub fn process_axis(&mut self, id: GamepadId, axis: Axis, value: f32) {
        for g in self.gamepads_mut().filter(|g| g.id == id) {
            g.axis(axis, value);
        }
    }

    fn gamepads_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut Gamepad> {
        self.devices.iter_mut().filter_map(|s| {
            s.as_mut().and_then(|s| match s {
                Device::Gamepad(k) => Some(k),
                _ => None,
            })
        })
    }

    fn keyboards_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut Keyboard> {
        self.devices.iter_mut().filter_map(|s| {
            s.as_mut().and_then(|s| match s {
                Device::Keyboard(k) => Some(k),
                _ => None,
            })
        })
    }
}

impl Debug for Handle {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

/// Binding configuration.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Bindings {
    keyboards: Vec<KeyboardBinding>,
    gamepads: HashMap<Uuid, GamepadBinding>,
}

impl Bindings {
    /// Gets a gamepad mapping, or inserts the default mapping into the config.
    pub fn get(&mut self, uuid: &Uuid) -> GamepadBinding {
        if let Some(bindings) = self.gamepads.get(uuid) {
            bindings.clone()
        } else {
            // insert default bindings
            self.gamepads.insert(uuid.clone(), Default::default());
            self.gamepads.get(uuid).unwrap().clone()
        }
    }
}

impl Default for Bindings {
    fn default() -> Bindings {
        Bindings {
            keyboards: vec![KeyboardBinding::default()],
            gamepads: HashMap::new(),
        }
    }
}

#[derive(Debug)]
enum Device {
    Keyboard(Keyboard),
    Gamepad(Gamepad),
}

impl Device {
    /// Samples a set of inputs.
    pub fn sample(&mut self) -> Inputs {
        match self {
            Device::Keyboard(k) => k.sample(),
            Device::Gamepad(g) => g.sample(),
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
    mapping: KeyboardBinding,
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
    pub fn new(mapping: KeyboardBinding) -> Keyboard {
        Keyboard {
            mapping,
            direction: 0,
            buttons: Buttons::empty(),
        }
    }

    /// Processes a key down event.
    pub fn key_down(&mut self, key: ScanCode) {
        if let Some(&direction) = self.mapping.direction_map.get(&key) {
            self.direction |= direction;
        }

        if let Some(&buttons) = self.mapping.button_map.get(&key) {
            self.buttons.insert(buttons);
        }
    }

    /// Processes a key up event.
    pub fn key_up(&mut self, key: ScanCode) {
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
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KeyboardBinding {
    direction_map: HashMap<ScanCode, u8>,
    button_map: HashMap<ScanCode, Buttons>,
}

impl Default for KeyboardBinding {
    fn default() -> KeyboardBinding {
        let mut direction_map = HashMap::new();

        direction_map.insert(0x11, DIRECTION_UP);
        direction_map.insert(0x1F, DIRECTION_DOWN);
        direction_map.insert(0x1E, DIRECTION_LEFT);
        direction_map.insert(0x20, DIRECTION_RIGHT);

        let mut button_map = HashMap::new();

        button_map.insert(0x16, Buttons::P);
        button_map.insert(0x17, Buttons::K);
        button_map.insert(0x18, Buttons::S);
        button_map.insert(0x19, Buttons::H);

        KeyboardBinding {
            direction_map,
            button_map,
        }
    }
}

/// A gamepad sampler.
#[derive(Debug)]
pub struct Gamepad {
    id: GamepadId,
    axis_x: f32,
    axis_y: f32,
    buttons: Buttons,

    uuid: Uuid,
    mapping: GamepadBinding,
}

impl Gamepad {
    /// Creates a new `Gamepad` sampler.
    pub fn new(id: GamepadId, uuid: Uuid, mapping: GamepadBinding) -> Gamepad {
        Gamepad {
            id,
            axis_x: 0.,
            axis_y: 0.,
            buttons: Buttons::default(),

            uuid,
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
                a if a > -157.5 && a <= -112.5 => Direction::D1,
                a if a > -112.5 && a <= -67.5 => Direction::D2,
                a if a > -67.5 && a <= -22.5 => Direction::D3,
                a if a > -22.5 && a <= 22.5 => Direction::D6,
                a if a > 22.5 && a <= 67.5 => Direction::D9,
                a if a > 67.5 && a <= 112.5 => Direction::D8,
                a if a > 112.5 && a <= 157.5 => Direction::D7,
                a if a > 157.5 && a <= 180.0 => Direction::D4,
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
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GamepadBinding {
    button_map: HashMap<Button, Buttons>,
    deadzone: f32,
}

impl Default for GamepadBinding {
    fn default() -> GamepadBinding {
        let mut button_map = HashMap::new();

        button_map.insert(Button::South, Buttons::K);
        button_map.insert(Button::West, Buttons::P);
        button_map.insert(Button::North, Buttons::S);
        button_map.insert(Button::East, Buttons::H);

        GamepadBinding {
            button_map,
            deadzone: 0.1,
        }
    }
}
