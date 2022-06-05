// Default character idle script.

let frames = 0;

fn onupdate() {
    frames += 1;

    if frames % 60 == 0 {
        print(`${frames / 60} second(s) have passed in idle animation!`)
    }

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

