use sotu::prelude::*;

pub enum Event {
    Next,
}

#[derive(Default)]
pub struct Linked;

#[state_machine(initial = State::member(0))]
impl Linked {
    #[state(superstate = group)]
    fn member(count: &mut u32, event: &Event) -> Outcome<State> {
        *count += 1;

        match event {
            Event::Next => Super,
        }
    }

    #[state]
    fn alone(event: &Event) -> Outcome<State> {
        match event {
            Event::Next => Transition(State::member(0)),
        }
    }

    #[superstate]
    fn group(count: &u32, event: &Event) -> Outcome<State> {
        match event {
            Event::Next if *count > 1 => Transition(State::alone()),
            Event::Next => Handled,
        }
    }
}

fn main() {
    let mut machine = Linked.state_machine();

    machine.handle(Event::Next, &mut ());
}
