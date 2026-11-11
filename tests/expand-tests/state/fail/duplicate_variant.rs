use sotu::prelude::*;

struct Lamp;

#[state_machine(initial = State::idle())]
impl Lamp {
    #[state]
    fn idle() -> Outcome<State> {
        Handled
    }

    #[state(name = Idle)]
    fn waiting() -> Outcome<State> {
        Handled
    }
}

fn main() {}
