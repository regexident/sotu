use sotu::prelude::*;

struct Lamp;

#[state_machine(initial = State::a(), bogus = 1)]
impl Lamp {
    #[state]
    fn a() -> Outcome<State> {
        Handled
    }
}

fn main() {}
