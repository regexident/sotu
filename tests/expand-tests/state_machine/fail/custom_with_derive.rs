use sotu::prelude::*;

struct Lamp;

#[state_machine(initial = State::on(), state(custom, derive(Debug)))]
impl Lamp {
    #[state]
    fn on() -> Outcome<State> {
        Handled
    }
}

fn main() {}
