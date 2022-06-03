// Default character idle script.

fn onupdate() {
    if state.flipped {
        if inputs.direction == D6 {
            state.change("backward")
        } else if inputs.direction == D4 {
            state.change("forward")
        }
    } else {
        if inputs.direction == D6 {
            state.change("forward")
        } else if inputs.direction == D4 {
            state.change("backward")
        }
    }
}

