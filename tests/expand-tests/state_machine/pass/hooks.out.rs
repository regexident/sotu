use sotu::prelude::*;
pub struct Tracer {
    pub trace: Vec<&'static str>,
}
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
    fn a() -> Outcome<State> {
        Transition(State::b())
    }
    fn b() -> Outcome<State> {
        Handled
    }
}
pub enum State {
    A {},
    B {},
}
impl State {
    pub fn a() -> Self {
        Self::A {}
    }
    pub fn b() -> Self {
        Self::B {}
    }
}
impl ::sotu::blocking::IntoStateMachine for Tracer {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = ();
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::a()
    }
    fn before_dispatch<'ctx, 'data: 'ctx>(
        &mut self,
        node: ::sotu::StateOrSuperstate<'_, State, ()>,
        event: &(),
        context: &mut (),
    ) {
        ::sotu::blocking::DispatchHook::call_dispatch_hook(
            &(Self::before_dispatch),
            self,
            node,
            event,
            context,
        )
    }
    fn after_dispatch<'ctx, 'data: 'ctx>(
        &mut self,
        node: ::sotu::StateOrSuperstate<'_, State, ()>,
        event: &(),
        context: &mut (),
    ) {
        ::sotu::blocking::DispatchHook::call_dispatch_hook(
            &(Self::after_dispatch),
            self,
            node,
            event,
            context,
        )
    }
    fn before_transition<'ctx, 'data: 'ctx>(
        &mut self,
        source: &State,
        target: &State,
        context: &mut (),
    ) {
        ::sotu::blocking::TransitionHook::call_transition_hook(
            &(Self::before_transition),
            self,
            source,
            target,
            context,
        )
    }
    fn after_transition<'ctx, 'data: 'ctx>(
        &mut self,
        source: &State,
        target: &State,
        context: &mut (),
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
        _context: &mut (),
    ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
        match self {
            State::A {} => Tracer::a(),
            State::B {} => Tracer::b(),
        }
    }
}
impl ::sotu::private::Submachine for Tracer {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = ();
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::a()
    }
}
impl ::sotu::private::BlockingSubmachine for Tracer {}
fn main() {
    let mut machine = Tracer { trace: Vec::new() }.state_machine();
    machine.step(&mut ());
}
