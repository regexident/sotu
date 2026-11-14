use sotu::prelude::*;
pub struct Sink<'ctx, 'data> {
    pub output: &'ctx mut Vec<&'data str>,
}
pub struct Tracer;
impl Tracer {
    fn before_dispatch(
        &mut self,
        _node: StateOrSuperstate<'_, State, Superstate>,
        _event: &(),
        sink: &mut Sink<'_, '_>,
    ) {
        sink.output.push("before dispatch");
    }
    fn after_transition(
        &mut self,
        _source: &State,
        _target: &State,
        sink: &mut Sink<'_, '_>,
    ) {
        sink.output.push("after transition");
    }
    fn a() -> Outcome<State> {
        Super
    }
    fn group() -> Outcome<State> {
        Transition(State::a())
    }
}
pub enum State {
    A {},
}
impl State {
    pub fn a() -> Self {
        Self::A {}
    }
}
pub enum Superstate {
    Group {},
}
impl ::sotu::blocking::IntoStateMachine for Tracer {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = Sink<'ctx, 'data>;
    type State = State;
    type Superstate<'sub> = Superstate where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::a()
    }
    fn before_dispatch<'ctx, 'data: 'ctx>(
        &mut self,
        node: ::sotu::StateOrSuperstate<'_, State, Superstate>,
        event: &(),
        context: &mut Sink<'ctx, 'data>,
    ) {
        ::sotu::blocking::DispatchHook::call_dispatch_hook(
            &(Self::before_dispatch),
            self,
            node,
            event,
            context,
        )
    }
    fn after_transition<'ctx, 'data: 'ctx>(
        &mut self,
        source: &State,
        target: &State,
        context: &mut Sink<'ctx, 'data>,
    ) {
        ::sotu::blocking::TransitionHook::call_transition_hook(
            &(Self::after_transition),
            self,
            source,
            target,
            context,
        )
    }
}
impl ::sotu::blocking::State<Tracer> for State {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Tracer,
        _event: &(),
        _context: &mut Sink<'ctx, 'data>,
    ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
        match self {
            State::A {} => Tracer::a(),
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate> {
        match self {
            State::A {} => ::core::option::Option::Some(Superstate::Group {}),
        }
    }
}
impl ::sotu::blocking::Superstate<Tracer> for Superstate {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Tracer,
        _event: &(),
        _context: &mut Sink<'ctx, 'data>,
    ) -> ::sotu::Outcome<State, ::core::convert::Infallible> {
        match self {
            Superstate::Group {} => Tracer::group(),
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate> {
        match self {
            Superstate::Group {} => ::core::option::Option::None,
        }
    }
    fn same_state(&self, other: &Superstate) -> bool {
        match (self, other) {
            (Superstate::Group { .. }, Superstate::Group { .. }) => true,
        }
    }
}
impl ::sotu::private::Submachine for Tracer {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = Sink<'ctx, 'data>;
    type State = State;
    type Superstate<'sub> = Superstate where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::a()
    }
}
impl ::sotu::private::BlockingSubmachine for Tracer {}
impl ::sotu::private::ExitlessSubmachine for Tracer {}
fn main() {
    let mut lines = Vec::new();
    let mut sink = Sink { output: &mut lines };
    let mut machine = Tracer.state_machine();
    machine.step(&mut sink);
}
