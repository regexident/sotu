use sotu::prelude::*;

#[derive(Default)]
pub struct Lamp;

#[state_machine(initial = State::off())]
impl Lamp {
    #[state]
    fn off(exit: &mut bool) -> Outcome<State> {
        *exit = true;

        Handled
    }
}

fn main() {}
