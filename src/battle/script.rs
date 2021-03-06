//! Scripting support for battles.

use rhai::{
    packages::{Package, StandardPackage},
    Module, Shared,
};
pub use rhai::{Scope, AST};

use super::fsm::Key;
use super::State;
use crate::input::{Buffer, Direction};

use std::ops::{Add, Deref, Div, Mul, Sub};

use glam::f32::Vec2;

/// A scripting engine.
pub struct Engine(rhai::Engine);

impl Engine {
    /// Creates a new engine.
    pub fn new() -> Engine {
        let mut engine = rhai::Engine::new_raw();

        let mut module = Module::new();

        module.set_var("D1", Direction::D1);
        module.set_var("D2", Direction::D2);
        module.set_var("D3", Direction::D3);
        module.set_var("D4", Direction::D4);
        module.set_var("D5", Direction::D5);
        module.set_var("D6", Direction::D6);
        module.set_var("D7", Direction::D7);
        module.set_var("D8", Direction::D8);
        module.set_var("D9", Direction::D9);

        let module: Shared<Module> = module.into();

        engine
            .set_max_expr_depths(0, 0)
            .register_global_module(module)
            .register_global_module(StandardPackage::new().as_shared_module())
            // Register on_print and on_debug
            .on_print(move |s| info!("{}", s))
            .on_debug(move |s, src, pos| debug!("{} @ {:?} > {}", src.unwrap_or("unknown"), pos, s))
            // Register some nonsense f32 functions.
            //.register_fn("-", f32::neg)
            // Vec2 impl
            .register_type::<Vec2>()
            .register_fn("vec2", Vec2::new)
            .register_get_set("x", |v: &mut Vec2| v.x, |v: &mut Vec2, x: f32| v.x = x)
            .register_get_set("y", |v: &mut Vec2| v.y, |v: &mut Vec2, y: f32| v.y = y)
            .register_fn("+", <Vec2 as Add<Vec2>>::add)
            .register_fn("-", <Vec2 as Sub<Vec2>>::sub)
            .register_fn("*", <Vec2 as Mul<Vec2>>::mul)
            .register_fn("/", <Vec2 as Div<Vec2>>::div)
            .register_fn("*", <Vec2 as Mul<f32>>::mul)
            .register_fn("*", <f32 as Mul<Vec2>>::mul)
            // Direction impl
            .register_type::<Direction>()
            .register_fn("==", |d1: Direction, d2: Direction| d1 == d2)
            .register_fn("!=", |d1: Direction, d2: Direction| d1 != d2)
            // Buffer impl
            .register_type::<Buffer>()
            .register_get("direction", |v: &mut Buffer| v.direction())
            .register_get("buttons", |v: &mut Buffer| v.buttons())
            // State impl
            .register_type::<State>()
            .register_get_set(
                "pos",
                |s: &mut State| s.pos,
                |s: &mut State, pos: Vec2| s.pos = pos,
            )
            .register_fn(
                "direction_x",
                |s: &mut State| {
                    if s.flipped {
                        -1.0f32
                    } else {
                        1.0f32
                    }
                },
            )
            .register_get("flipped", |s: &mut State| s.flipped)
            .register_fn("change", |s: &mut State, name: &str| {
                s.key = Key::from(name)
            });

        Engine(engine)
    }
}

impl Deref for Engine {
    type Target = rhai::Engine;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
