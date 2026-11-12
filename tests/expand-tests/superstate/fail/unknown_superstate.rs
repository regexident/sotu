use sotu::prelude::*;

struct Lamp;

#[state_machine(initial = State::on())]
impl Lamp {
    #[state(superstate = group)]
    fn on() -> Outcome<State> {
        Super
    }

    #[superstate(superstate = missing)]
    fn group() -> Outcome<State> {
        Super
    }
}

fn main() {}
