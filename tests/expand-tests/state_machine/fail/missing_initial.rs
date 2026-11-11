use sotu::prelude::*;

struct Lamp;

#[state_machine]
impl Lamp {
    #[state]
    fn a() -> Outcome<State> {
        Handled
    }
}

fn main() {}
