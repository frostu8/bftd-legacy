// Default character idle script.

fn onupdate() {
    if inputs.direction() == DIRECTION6 {
        switch_state("forward")
    }
}

