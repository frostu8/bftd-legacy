//! Input sampling utilities.
//!
//! An input sampler is to gather events from the OS about connected controllers
//! and keyboards, and convert those inputs into a platform-independent version.
//!
//! An input sampler should also prevent "input tunneling," meaning if a button
//! is down, then up within a frame, the input sampler should sample 1 frame of
//! that button being pressed.

pub mod keyboard;

pub use keyboard::Keyboard;

