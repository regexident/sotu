use sotu::prelude::*;

pub struct Tracer {
    pub trace: Vec<&'static str>,
}

#[state_machine(
    initial = State::a(),
    before_dispatch = Self::before_dispatch,
    after_dispatch = Self::after_dispatch,
    before_transition = Self::before_transition,
    after_transition = Self::after_transition
)]
impl Tracer {
    fn before_dispatch(&mut self, _node: StateOrSuperstate<'_, State, ()>, _event: &()) {
        self.trace.push("before dispatch");
    }

    fn after_dispatch(&mut self, _node: StateOrSuperstate<'_, State, ()>, _event: &()) {
        self.trace.push("after dispatch");
    }

    fn before_transition(&mut self, _source: &State, _target: &State) {
        self.trace.push("before transition");
    }

    fn after_transition(&mut self, _source: &State, _target: &State) {
        self.trace.push("after transition");
    }

    #[state]
    fn a() -> Outcome<State> {
        Transition(State::b())
    }

    #[state]
    fn b() -> Outcome<State> {
        Handled
    }
}

fn main() {
    let mut machine = Tracer { trace: Vec::new() }.state_machine();

    machine.step(&mut ());
}
