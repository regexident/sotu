use sotu::prelude::*;

pub enum Event {
    Next,
}

#[derive(Default)]
pub struct Hierarchy;

#[state_machine(initial = State::leaf())]
impl Hierarchy {
    /// The state below every superstate.
    #[state(superstate = inner)]
    fn leaf(event: &Event) -> Outcome<State> {
        match event {
            Event::Next => Super,
        }
    }

    #[state]
    fn top(event: &Event) -> Outcome<State> {
        match event {
            Event::Next => Transition(State::leaf()),
        }
    }

    /// The nearest superstate of `leaf`.
    #[superstate(superstate = outer)]
    fn inner(event: &Event) -> Outcome<State> {
        match event {
            Event::Next => Super,
        }
    }

    /// The outermost superstate, renamed.
    #[superstate(name = Outermost)]
    fn outer(event: &Event) -> Outcome<State> {
        match event {
            Event::Next => Transition(State::top()),
        }
    }
}

fn main() {
    let mut machine = Hierarchy.state_machine();

    machine.handle(Event::Next, &mut ());
}
