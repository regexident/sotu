use sotu::prelude::*;

struct Lamp;

#[state_machine(initial = "State::a()")]
impl Lamp {
    #[state]
    fn a() -> Outcome<State> {
        Handled
    }
}

fn main() {}
