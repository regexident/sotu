use sotu::prelude::*;

struct Lamp;

#[state_machine(initial = State::on())]
impl Lamp {
    #[state(superstate = first)]
    fn on() -> Outcome<State> {
        Super
    }

    #[superstate(superstate = second)]
    fn first() -> Outcome<State> {
        Super
    }

    #[superstate(superstate = first)]
    fn second() -> Outcome<State> {
        Super
    }
}

fn main() {}
