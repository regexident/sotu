use sotu::prelude::*;

pub enum Event {
    Next,
}

struct Lamp;

#[state_machine(initial = State::on())]
impl Lamp {
    #[state]
    fn on(#[default] event: &Event) -> Outcome<State> {
        match event {
            Event::Next => Handled,
        }
    }
}

fn main() {}
