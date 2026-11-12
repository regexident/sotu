use sotu::prelude::*;

struct Lamp;

#[state_machine(initial = State::on())]
impl Lamp {
    #[state(exit_action = cleanup)]
    fn on() -> Outcome<State> {
        Handled
    }
}

fn main() {}
