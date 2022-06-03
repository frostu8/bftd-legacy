// A state script that moves the character forward during the active state.

const MOVEMENT_PER_SECOND = 700.0 / 60.0;

fn onupdate() {
    if state.flipped {
        if inputs.direction == D4 {
            // move forward
            state.pos += vec2(global::MOVEMENT_PER_SECOND * state.direction_x(), 0.0)
        } else if inputs.direction == D6 {
            state.change("backward")
        } else {
            state.change("idle")
        }
    } else {
        if inputs.direction == D6 {
            // move forward
            state.pos += vec2(global::MOVEMENT_PER_SECOND * state.direction_x(), 0.0)
        } else if inputs.direction == D4 {
            state.change("backward")
        } else {
            state.change("idle")
        }
    }
}

