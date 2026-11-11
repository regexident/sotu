use sotu::prelude::*;

struct Counter;

#[state_machine(initial = State::on(0))]
impl Counter {
    #[state]
    fn on(counter: u32) -> Outcome<State> {
        Handled
    }
}

fn main() {}
