use sotu::prelude::*;

struct Lamp;

#[state_machine(initial = State::on())]
impl Lamp {
    #[state]
    fn on((low, high): &(u8, u8)) -> Outcome<State> {
        Handled
    }
}

fn main() {}
