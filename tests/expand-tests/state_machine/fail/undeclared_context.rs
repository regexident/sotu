use sotu::prelude::*;

struct Lamp;

#[state_machine(initial = State::on())]
impl Lamp {
    #[state]
    fn on(context: &mut u32) -> Outcome<State> {
        Handled
    }

    #[state]
    fn off(context: &mut u32) -> Outcome<State> {
        Handled
    }
}

fn main() {}
