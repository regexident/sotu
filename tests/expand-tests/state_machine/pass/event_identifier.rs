use sotu::prelude::*;

pub struct Flip;

#[derive(Event)]
pub enum Event {
    Flip(Flip),
}

#[derive(Default)]
pub struct Switch;

#[state_machine(initial = State::off(), event_identifier = input)]
impl Switch {
    #[state]
    fn off(input: &Event) -> Outcome<State> {
        match input {
            Event::Flip(_) => Transition(State::on()),
        }
    }

    #[state]
    fn on(input: &Event, #[default] event: &mut u32) -> Outcome<State> {
        *event += 1;

        match input {
            Event::Flip(_) => Transition(State::off()),
        }
    }
}

fn main() {
    let mut machine = Switch.state_machine();

    machine.handle(Flip, &mut ());
}
