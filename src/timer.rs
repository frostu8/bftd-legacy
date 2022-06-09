//! Time and frame-limiting utilities.

use std::time::{Duration, Instant};

/// A frame limiter
#[derive(Default)]
pub struct FrameLimiter {
    last: Option<Instant>,
}

impl FrameLimiter {
    /// Creates a new frame limiter.
    pub fn new() -> FrameLimiter {
        FrameLimiter::default()
    }

    /// Checks if the frame should be updated.
    pub fn should_update(&mut self, target_fps: u64) -> bool {
        let now = Instant::now();
        let time_between = Duration::from_millis(1000 / target_fps);

        if let Some(last) = &mut self.last {
            if now.duration_since(*last) >= time_between {
                *last = now;
                true
            } else {
                false
            }
        } else {
            self.last = Some(now);
            true
        }
    }
}
