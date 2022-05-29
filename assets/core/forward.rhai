// A state script that moves the character forward during the active state.

const MOVEMENT_PER_SECOND = 20.0 / 60.0;

fn onupdate() {
    if inputs().direction() != DIRECTION6 {
        switch_state("idle")
    } else {
        move(vec2(MOVEMENT_PER_SECOND, 0.0))
    }
}

