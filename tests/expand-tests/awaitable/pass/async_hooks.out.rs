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
    fn a() -> Outcome<State> {
        Transition(State::b())
    }
    fn b(context: &mut Vec<&'static str>) -> Outcome<State> {
        context.push("b");
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
impl ::sotu::awaitable::IntoStateMachine for Tracer {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = Vec<&'static str>;
    type State = State;
    type Superstate<'sub> = () where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::a()
    }
    fn before_dispatch<'ctx, 'data: 'ctx>(
        &mut self,
        node: ::sotu::StateOrSuperstate<
            '_,
            State,
            <Self as ::sotu::awaitable::IntoStateMachine>::Superstate<'_>,
        >,
        event: &(),
        context: &mut Vec<&'static str>,
    ) -> impl ::core::future::Future<Output = ()> {
        ::sotu::awaitable::DispatchHook::call_dispatch_hook(
            &(Self::before_dispatch),
            self,
            node,
            event,
            context,
        )
    }
    fn after_dispatch<'ctx, 'data: 'ctx>(
        &mut self,
        node: ::sotu::StateOrSuperstate<
            '_,
            State,
            <Self as ::sotu::awaitable::IntoStateMachine>::Superstate<'_>,
        >,
        event: &(),
        context: &mut Vec<&'static str>,
    ) -> impl ::core::future::Future<Output = ()> {
        ::sotu::awaitable::DispatchHook::call_dispatch_hook(
            &(after_dispatch),
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
        context: &mut Vec<&'static str>,
    ) -> impl ::core::future::Future<Output = ()> {
        ::sotu::awaitable::TransitionHook::call_transition_hook(
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
        context: &mut Vec<&'static str>,
    ) -> impl ::core::future::Future<Output = ()> {
        ::sotu::awaitable::TransitionHook::call_transition_hook(
            &(Self::after_transition),
            self,
            source,
            target,
            context,
        )
    }
}
impl ::sotu::awaitable::State<Tracer> for State {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Tracer,
        _event: &(),
        context: &mut Vec<&'static str>,
    ) -> impl ::core::future::Future<
        Output = ::sotu::Outcome<Self, ::core::convert::Infallible>,
    > {
        async move {
            match self {
                State::A {} => Tracer::a(),
                State::B {} => Tracer::b(context),
            }
        }
    }
}
impl ::sotu::private::Submachine for Tracer {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = Vec<&'static str>;
    type State = State;
    type Superstate<'sub> = () where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::a()
    }
}
impl ::sotu::private::ExitlessSubmachine for Tracer {}
fn main() {
    let _machine = Tracer { trace: Vec::new() }.state_machine();
}
