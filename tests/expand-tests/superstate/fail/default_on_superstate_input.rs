use sotu::prelude::*;

struct Counter;

#[state_machine(initial = State::on(0))]
impl Counter {
    #[state(superstate = group)]
    fn on(counter: &mut u32) -> Outcome<State> {
        Super
    }

    #[superstate]
    fn group(#[default] counter: &u32) -> Outcome<State> {
        Handled
    }
}

fn main() {}
