use sotu::prelude::*;

struct Lamp;

#[state_machine(initial = State::on())]
impl Lamp {
    #[state(entry_action = prepare)]
    fn on() -> Outcome<State> {
        Handled
    }
}

fn main() {}
