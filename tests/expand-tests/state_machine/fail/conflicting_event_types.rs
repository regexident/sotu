use sotu::prelude::*;

pub enum Event {
    Next,
}

pub enum Other {
    Next,
}

struct Lamp;

#[state_machine(initial = State::on())]
impl Lamp {
    #[state]
    fn on(event: &Event) -> Outcome<State> {
        Handled
    }

    #[state]
    fn off(event: &Other) -> Outcome<State> {
        Handled
    }
}

fn main() {}
