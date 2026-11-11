use sotu::prelude::*;

struct Lamp;

#[state_machine(initial = State::on())]
impl Lamp {
    #[state]
    fn on<T>() -> Outcome<State> {
        Handled
    }
}

fn main() {}
