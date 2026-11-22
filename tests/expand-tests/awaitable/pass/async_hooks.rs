use sotu::prelude::*;

pub struct Tracer {
    pub trace: Vec<&'static str>,
}

async fn after_dispatch(
    tracer: &mut Tracer,
    _node: StateOrSuperstate<'_, State, ()>,
    _event: &(),
) {
    tracer.trace.push("after dispatch");
}

#[state_machine(
    initial = State::a(),
    context = Vec<&'static str>,
    before_dispatch = Self::before_dispatch,
    after_dispatch = after_dispatch,
    before_transition = Self::before_transition,
    after_transition = Self::after_transition
)]
impl Tracer {
    fn before_dispatch(&mut self, _node: StateOrSuperstate<'_, State, ()>, _event: &()) {
        self.trace.push("before dispatch");
    }

    async fn before_transition(
        &mut self,
        _source: &State,
        _target: &State,
        context: &mut Vec<&'static str>,
    ) {
        context.push("before transition");
    }

    fn after_transition(&mut self, _source: &State, _target: &State) {
        self.trace.push("after transition");
    }

    #[state]
    fn a() -> Outcome<State> {
        Transition(State::b())
    }

    #[state]
    fn b(context: &mut Vec<&'static str>) -> Outcome<State> {
        context.push("b");

        Handled
    }
}

fn main() {
    let _machine = Tracer { trace: Vec::new() }.state_machine();
}
