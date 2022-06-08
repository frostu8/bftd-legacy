// A state script that moves the character backward during the active state.

const MOVEMENT_PER_SECOND = 2.0 / 60.0;

fn onupdate() {
    if state.flipped {
        if inputs.direction == D6 {
            // move backward
            state.pos += vec2(-global::MOVEMENT_PER_SECOND * state.direction_x(), 0.0)
        } else if inputs.direction == D4 {
            state.change("forward")
        } else {
            state.change("idle")
        }
    } else {
        if inputs.direction == D4 {
            // move backward
            state.pos += vec2(-global::MOVEMENT_PER_SECOND * state.direction_x(), 0.0)
        } else if inputs.direction == D6 {
            state.change("forward")
        } else {
            state.change("idle")
        }
    }
}

