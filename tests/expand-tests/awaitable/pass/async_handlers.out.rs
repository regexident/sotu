use sotu::prelude::*;
pub enum Event {
    Tick,
    Reset,
}
pub struct Blinker {
    pub ticks: u32,
}
#[automatically_derived]
impl ::core::default::Default for Blinker {
    #[inline]
    fn default() -> Blinker {
        Blinker {
            ticks: ::core::default::Default::default(),
        }
    }
}
impl Blinker {
    async fn outer(&mut self, counter: &u32, event: &Event) -> Outcome<State> {
        match event {
            Event::Tick => {
                self.ticks = *counter;
                Handled
            }
            Event::Reset => Transition(State::off()),
        }
    }
    async fn blinking(counter: &mut u32, event: &Event) -> Outcome<State> {
        match event {
            Event::Tick => {
                *counter += 1;
                Super
            }
            Event::Reset => Super,
        }
    }
    async fn on(counter: &mut u32, event: &Event) -> Outcome<State> {
        match event {
            Event::Tick if *counter > 3 => Transition(State::off()),
            Event::Tick | Event::Reset => Super,
        }
    }
    fn off(event: &Event) -> Outcome<State> {
        match event {
            Event::Tick => Handled,
            Event::Reset => Transition(State::on(0)),
        }
    }
}
pub enum State {
    On { counter: u32 },
    Off {},
}
impl State {
    pub fn on(counter: u32) -> Self {
        Self::On { counter }
    }
    pub fn off() -> Self {
        Self::Off {}
    }
}
pub enum Superstate<'sub> {
    Outer { counter: &'sub u32 },
    Blinking { counter: &'sub mut u32 },
}
impl ::sotu::awaitable::IntoStateMachine for Blinker {
    type Event = Event;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::on(0)
    }
}
impl ::sotu::awaitable::State<Blinker> for State {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Blinker,
        event: &Event,
        _context: &mut (),
    ) -> impl ::core::future::Future<
        Output = ::sotu::Outcome<Self, ::core::convert::Infallible>,
    > {
        async move {
            match self {
                State::On { counter } => Blinker::on(counter, event).await,
                State::Off {} => Blinker::off(event),
            }
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate<'_>> {
        match self {
            State::On { counter } => {
                ::core::option::Option::Some(Superstate::Blinking { counter })
            }
            State::Off {} => ::core::option::Option::None,
        }
    }
}
impl<'sub> ::sotu::awaitable::Superstate<Blinker> for Superstate<'sub> {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Blinker,
        event: &Event,
        _context: &mut (),
    ) -> impl ::core::future::Future<
        Output = ::sotu::Outcome<State, ::core::convert::Infallible>,
    > {
        async move {
            match self {
                Superstate::Outer { counter } => {
                    Blinker::outer(shared_storage, counter, event).await
                }
                Superstate::Blinking { counter } => {
                    Blinker::blinking(counter, event).await
                }
            }
        }
    }
    fn superstate(self) -> ::core::option::Option<Self> {
        match self {
            Superstate::Outer { .. } => ::core::option::Option::None,
            Superstate::Blinking { counter } => {
                ::core::option::Option::Some(Superstate::Outer { counter })
            }
        }
    }
    fn same_state(&self, other: &Superstate<'_>) -> bool {
        match (self, other) {
            (Superstate::Outer { .. }, Superstate::Outer { .. }) => true,
            (Superstate::Blinking { .. }, Superstate::Blinking { .. }) => true,
            _ => false,
        }
    }
}
impl ::sotu::private::Submachine for Blinker {
    type Event = Event;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::on(0)
    }
}
impl ::sotu::private::ExitlessSubmachine for Blinker {}
impl ::sotu::private::AwaitableSubmachine for Blinker {
    fn state_superstate<'sub>(
        state: &'sub mut <Self as ::sotu::private::Submachine>::State,
    ) -> ::core::option::Option<<Self as ::sotu::private::Submachine>::Superstate<'sub>>
    where
        Self: 'sub,
    {
        <<Blinker as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
            Blinker,
        >>::superstate(state)
    }
    fn state_same(
        first: &<Self as ::sotu::private::Submachine>::State,
        second: &<Self as ::sotu::private::Submachine>::State,
    ) -> bool {
        <<Blinker as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
            Blinker,
        >>::same_state(first, second)
    }
    fn superstate_same<'first, 'second>(
        first: &<Self as ::sotu::private::Submachine>::Superstate<'first>,
        second: &<Self as ::sotu::private::Submachine>::Superstate<'second>,
    ) -> bool
    where
        Self: 'first + 'second,
    {
        <<Blinker as ::sotu::awaitable::IntoStateMachine>::Superstate<
            '_,
        > as ::sotu::awaitable::Superstate<Blinker>>::same_state(first, second)
    }
    fn state_depth(state: &mut <Self as ::sotu::private::Submachine>::State) -> usize {
        <<Blinker as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::StateExt<
            Blinker,
        >>::depth(state)
    }
    fn state_is_addressed(
        state: &<Self as ::sotu::private::Submachine>::State,
        event: <Self as ::sotu::private::Submachine>::Event,
    ) -> (bool, <Self as ::sotu::private::Submachine>::Event) {
        <<Blinker as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
            Blinker,
        >>::__is_addressed(state, event)
    }
    fn superstate_is_addressed<'sub>(
        superstate: &<Self as ::sotu::private::Submachine>::Superstate<'sub>,
        event: <Self as ::sotu::private::Submachine>::Event,
    ) -> (bool, <Self as ::sotu::private::Submachine>::Event)
    where
        Self: 'sub,
    {
        <<Blinker as ::sotu::awaitable::IntoStateMachine>::Superstate<
            '_,
        > as ::sotu::awaitable::Superstate<Blinker>>::__is_addressed(superstate, event)
    }
    fn superstate_depth<'sub>(
        superstate: &<Self as ::sotu::private::Submachine>::Superstate<'sub>,
    ) -> usize
    where
        Self: 'sub,
    {
        superstate.__depth()
    }
    fn superstate_parent<'sub>(
        superstate: <Self as ::sotu::private::Submachine>::Superstate<'sub>,
    ) -> ::core::option::Option<<Self as ::sotu::private::Submachine>::Superstate<'sub>>
    where
        Self: 'sub,
    {
        <<Blinker as ::sotu::awaitable::IntoStateMachine>::Superstate<
            '_,
        > as ::sotu::awaitable::Superstate<Blinker>>::superstate(superstate)
    }
    fn dispatch_state<'ctx, 'data: 'ctx>(
        state: &mut <Self as ::sotu::private::Submachine>::State,
        shared_storage: &mut Self,
        event: <Self as ::sotu::private::Submachine>::Event,
        context: &mut <Self as ::sotu::private::Submachine>::Context<'ctx, 'data>,
    ) -> impl ::core::future::Future<
        Output = ::sotu::private::Dispatched<
            <Self as ::sotu::private::Submachine>::State,
            <Self as ::sotu::private::Submachine>::Exit,
            <Self as ::sotu::private::Submachine>::Event,
        >,
    > {
        <<Blinker as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
            Blinker,
        >>::__dispatch(state, shared_storage, event, context)
    }
    fn dispatch_superstate<'ctx, 'data: 'ctx, 'sub>(
        superstate: &mut <Self as ::sotu::private::Submachine>::Superstate<'sub>,
        shared_storage: &mut Self,
        event: <Self as ::sotu::private::Submachine>::Event,
        context: &mut <Self as ::sotu::private::Submachine>::Context<'ctx, 'data>,
    ) -> impl ::core::future::Future<
        Output = ::sotu::private::Dispatched<
            <Self as ::sotu::private::Submachine>::State,
            <Self as ::sotu::private::Submachine>::Exit,
            <Self as ::sotu::private::Submachine>::Event,
        >,
    >
    where
        Self: 'sub,
    {
        <<Blinker as ::sotu::awaitable::IntoStateMachine>::Superstate<
            '_,
        > as ::sotu::awaitable::Superstate<
            Blinker,
        >>::__dispatch(superstate, shared_storage, event, context)
    }
    fn enter_state<'ctx, 'data: 'ctx>(
        state: &mut <Self as ::sotu::private::Submachine>::State,
        shared_storage: &mut Self,
        context: &mut <Self as ::sotu::private::Submachine>::Context<'ctx, 'data>,
    ) -> impl ::core::future::Future<Output = ()> {
        <<Blinker as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
            Blinker,
        >>::call_entry_action(state, shared_storage, context)
    }
    fn exit_state<'ctx, 'data: 'ctx>(
        state: &mut <Self as ::sotu::private::Submachine>::State,
        shared_storage: &mut Self,
        context: &mut <Self as ::sotu::private::Submachine>::Context<'ctx, 'data>,
    ) -> impl ::core::future::Future<Output = ()> {
        <<Blinker as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
            Blinker,
        >>::call_exit_action(state, shared_storage, context)
    }
    fn enter_superstate<'ctx, 'data: 'ctx, 'sub>(
        superstate: &mut <Self as ::sotu::private::Submachine>::Superstate<'sub>,
        shared_storage: &mut Self,
        context: &mut <Self as ::sotu::private::Submachine>::Context<'ctx, 'data>,
    ) -> impl ::core::future::Future<Output = ()>
    where
        Self: 'sub,
    {
        <<Blinker as ::sotu::awaitable::IntoStateMachine>::Superstate<
            '_,
        > as ::sotu::awaitable::Superstate<
            Blinker,
        >>::call_entry_action(superstate, shared_storage, context)
    }
    fn exit_superstate<'ctx, 'data: 'ctx, 'sub>(
        superstate: &mut <Self as ::sotu::private::Submachine>::Superstate<'sub>,
        shared_storage: &mut Self,
        context: &mut <Self as ::sotu::private::Submachine>::Context<'ctx, 'data>,
    ) -> impl ::core::future::Future<Output = ()>
    where
        Self: 'sub,
    {
        <<Blinker as ::sotu::awaitable::IntoStateMachine>::Superstate<
            '_,
        > as ::sotu::awaitable::Superstate<
            Blinker,
        >>::call_exit_action(superstate, shared_storage, context)
    }
}
impl<'sub> Superstate<'sub> {
    fn __depth(&self) -> usize {
        match self {
            Superstate::Outer { .. } => 1usize,
            Superstate::Blinking { .. } => 2usize,
        }
    }
}
fn main() {
    let _machine = Blinker::default().state_machine();
}
