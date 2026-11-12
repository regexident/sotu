use sotu::prelude::*;

struct Counter;

#[state_machine(initial = State::on())]
impl Counter {
    #[state(superstate = group)]
    fn on() -> Outcome<State> {
        Super
    }

    #[superstate]
    fn group(counter: &u32) -> Outcome<State> {
        Handled
    }
}

fn main() {}
