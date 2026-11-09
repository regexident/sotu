use sotu::prelude::*;

pub struct Sink<'ctx, 'data> {
    pub output: &'ctx mut Vec<&'data str>,
}

pub struct Tracer;

#[state_machine(
    initial = State::a(),
    context = Sink<'ctx, 'data>,
    before_dispatch = Self::before_dispatch,
    after_transition = Self::after_transition
)]
impl Tracer {
    fn before_dispatch(
        &mut self,
        _node: StateOrSuperstate<'_, State, Superstate>,
        _event: &(),
        sink: &mut Sink<'_, '_>,
    ) {
        sink.output.push("before dispatch");
    }

    fn after_transition(&mut self, _source: &State, _target: &State, sink: &mut Sink<'_, '_>) {
        sink.output.push("after transition");
    }

    #[state(superstate = group)]
    fn a() -> Outcome<State> {
        Super
    }

    #[superstate]
    fn group() -> Outcome<State> {
        Transition(State::a())
    }
}

fn main() {
    let mut lines = Vec::new();
    let mut sink = Sink {
        output: &mut lines,
    };

    let mut machine = Tracer.state_machine();

    machine.step(&mut sink);
}
