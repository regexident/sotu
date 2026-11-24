use sotu::prelude::*;
pub enum Event {
    Flip,
}
pub struct Lamp {
    pub flips: u32,
}
#[automatically_derived]
impl ::core::default::Default for Lamp {
    #[inline]
    fn default() -> Lamp {
        Lamp {
            flips: ::core::default::Default::default(),
        }
    }
}
impl Lamp {
    fn powered() -> Outcome<State> {
        Super
    }
    fn on(lit: &mut bool, event: &Event) -> Outcome<State> {
        match event {
            Event::Flip if *lit => Transition(State::off()),
            Event::Flip => Handled,
        }
    }
    fn off(event: &Event) -> Outcome<State> {
        match event {
            Event::Flip => Transition(State::on(false)),
        }
    }
    async fn power_up(&mut self) {
        self.flips += 1;
    }
    fn power_down(&mut self) {
        self.flips += 1;
    }
    async fn light(&mut self, lit: &mut bool) {
        *lit = true;
        self.flips += 1;
    }
    fn dim(lit: &mut bool) {
        *lit = false;
    }
}
pub enum State {
    On { lit: bool },
    Off {},
}
impl State {
    pub fn on(lit: bool) -> Self {
        Self::On { lit }
    }
    pub fn off() -> Self {
        Self::Off {}
    }
}
pub enum Superstate {
    Powered {},
}
impl ::sotu::awaitable::IntoStateMachine for Lamp {
    type Event = Event;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Superstate where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::off()
    }
}
impl ::sotu::awaitable::State<Lamp> for State {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Lamp,
        event: &Event,
        _context: &mut (),
    ) -> impl ::core::future::Future<
        Output = ::sotu::Outcome<Self, ::core::convert::Infallible>,
    > {
        async move {
            match self {
                State::On { lit } => Lamp::on(lit, event),
                State::Off {} => Lamp::off(event),
            }
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Lamp,
        _context: &mut (),
    ) -> impl ::core::future::Future<Output = ()> {
        async move {
            match self {
                State::On { lit } => Lamp::light(shared_storage, lit).await,
                State::Off {} => {}
            }
        }
    }
    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Lamp,
        _context: &mut (),
    ) -> impl ::core::future::Future<Output = ()> {
        async move {
            match self {
                State::On { lit } => Lamp::dim(lit),
                State::Off {} => {}
            }
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate> {
        match self {
            State::On { .. } => ::core::option::Option::Some(Superstate::Powered {}),
            State::Off {} => ::core::option::Option::None,
        }
    }
}
impl ::sotu::awaitable::Superstate<Lamp> for Superstate {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Lamp,
        _event: &Event,
        _context: &mut (),
    ) -> impl ::core::future::Future<
        Output = ::sotu::Outcome<State, ::core::convert::Infallible>,
    > {
        async move {
            match self {
                Superstate::Powered {} => Lamp::powered(),
            }
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Lamp,
        _context: &mut (),
    ) -> impl ::core::future::Future<Output = ()> {
        async move {
            match self {
                Superstate::Powered {} => Lamp::power_up(shared_storage).await,
            }
        }
    }
    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Lamp,
        _context: &mut (),
    ) -> impl ::core::future::Future<Output = ()> {
        async move {
            match self {
                Superstate::Powered {} => Lamp::power_down(shared_storage),
            }
        }
    }
    fn superstate(self) -> ::core::option::Option<Self> {
        match self {
            Superstate::Powered {} => ::core::option::Option::None,
        }
    }
    fn same_state(&self, other: &Superstate) -> bool {
        match (self, other) {
            (Superstate::Powered { .. }, Superstate::Powered { .. }) => true,
        }
    }
}
impl ::sotu::private::Submachine for Lamp {
    type Event = Event;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Superstate where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::off()
    }
}
impl ::sotu::private::ExitlessSubmachine for Lamp {}
impl ::sotu::private::AwaitableSubmachine for Lamp {
    fn state_superstate<'sub>(
        state: &'sub mut <Self as ::sotu::private::Submachine>::State,
    ) -> ::core::option::Option<<Self as ::sotu::private::Submachine>::Superstate<'sub>>
    where
        Self: 'sub,
    {
        <<Lamp as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
            Lamp,
        >>::superstate(state)
    }
    fn state_same(
        first: &<Self as ::sotu::private::Submachine>::State,
        second: &<Self as ::sotu::private::Submachine>::State,
    ) -> bool {
        <<Lamp as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
            Lamp,
        >>::same_state(first, second)
    }
    fn superstate_same<'first, 'second>(
        first: &<Self as ::sotu::private::Submachine>::Superstate<'first>,
        second: &<Self as ::sotu::private::Submachine>::Superstate<'second>,
    ) -> bool
    where
        Self: 'first + 'second,
    {
        <<Lamp as ::sotu::awaitable::IntoStateMachine>::Superstate<
            '_,
        > as ::sotu::awaitable::Superstate<Lamp>>::same_state(first, second)
    }
    fn state_depth(state: &mut <Self as ::sotu::private::Submachine>::State) -> usize {
        <<Lamp as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::StateExt<
            Lamp,
        >>::depth(state)
    }
    fn state_is_addressed(
        state: &<Self as ::sotu::private::Submachine>::State,
        event: <Self as ::sotu::private::Submachine>::Event,
    ) -> (bool, <Self as ::sotu::private::Submachine>::Event) {
        <<Lamp as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
            Lamp,
        >>::__is_addressed(state, event)
    }
    fn superstate_is_addressed<'sub>(
        superstate: &<Self as ::sotu::private::Submachine>::Superstate<'sub>,
        event: <Self as ::sotu::private::Submachine>::Event,
    ) -> (bool, <Self as ::sotu::private::Submachine>::Event)
    where
        Self: 'sub,
    {
        <<Lamp as ::sotu::awaitable::IntoStateMachine>::Superstate<
            '_,
        > as ::sotu::awaitable::Superstate<Lamp>>::__is_addressed(superstate, event)
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
        <<Lamp as ::sotu::awaitable::IntoStateMachine>::Superstate<
            '_,
        > as ::sotu::awaitable::Superstate<Lamp>>::superstate(superstate)
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
        <<Lamp as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
            Lamp,
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
        <<Lamp as ::sotu::awaitable::IntoStateMachine>::Superstate<
            '_,
        > as ::sotu::awaitable::Superstate<
            Lamp,
        >>::__dispatch(superstate, shared_storage, event, context)
    }
    fn enter_state<'ctx, 'data: 'ctx>(
        state: &mut <Self as ::sotu::private::Submachine>::State,
        shared_storage: &mut Self,
        context: &mut <Self as ::sotu::private::Submachine>::Context<'ctx, 'data>,
    ) -> impl ::core::future::Future<Output = ()> {
        <<Lamp as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
            Lamp,
        >>::call_entry_action(state, shared_storage, context)
    }
    fn exit_state<'ctx, 'data: 'ctx>(
        state: &mut <Self as ::sotu::private::Submachine>::State,
        shared_storage: &mut Self,
        context: &mut <Self as ::sotu::private::Submachine>::Context<'ctx, 'data>,
    ) -> impl ::core::future::Future<Output = ()> {
        <<Lamp as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
            Lamp,
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
        <<Lamp as ::sotu::awaitable::IntoStateMachine>::Superstate<
            '_,
        > as ::sotu::awaitable::Superstate<
            Lamp,
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
        <<Lamp as ::sotu::awaitable::IntoStateMachine>::Superstate<
            '_,
        > as ::sotu::awaitable::Superstate<
            Lamp,
        >>::call_exit_action(superstate, shared_storage, context)
    }
}
impl Superstate {
    fn __depth(&self) -> usize {
        match self {
            Superstate::Powered { .. } => 1usize,
        }
    }
}
fn main() {
    let _machine = Lamp::default().state_machine();
}
