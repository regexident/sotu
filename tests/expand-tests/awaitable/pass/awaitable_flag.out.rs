use sotu::prelude::*;
pub enum Event {
    Press,
    Finish,
}
pub enum Done {
    Finished,
}
pub struct Button {
    pub presses: u32,
}
#[automatically_derived]
impl ::core::default::Default for Button {
    #[inline]
    fn default() -> Button {
        Button {
            presses: ::core::default::Default::default(),
        }
    }
}
impl Button {
    fn idle(event: &Event) -> Outcome<State, Done> {
        match event {
            Event::Press => Transition(State::idle()),
            Event::Finish => Exit(Done::Finished),
        }
    }
    fn count(&mut self) {
        self.presses += 1;
    }
}
pub enum State {
    Idle {},
}
impl State {
    pub fn idle() -> Self {
        Self::Idle {}
    }
}
impl ::sotu::awaitable::IntoStateMachine for Button {
    type Event = Event;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = () where Self: 'sub;
    type Exit = Done;
    fn initial() -> State {
        State::idle()
    }
}
impl ::sotu::awaitable::State<Button> for State {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Button,
        event: &Event,
        _context: &mut (),
    ) -> impl ::core::future::Future<Output = ::sotu::Outcome<Self, Done>> {
        async move {
            match self {
                State::Idle {} => Button::idle(event),
            }
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Button,
        _context: &mut (),
    ) -> impl ::core::future::Future<Output = ()> {
        async move {
            match self {
                State::Idle {} => Button::count(shared_storage),
            }
        }
    }
}
impl ::sotu::private::Submachine for Button {
    type Event = Event;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = () where Self: 'sub;
    type Exit = Done;
    fn initial() -> State {
        State::idle()
    }
}
impl ::sotu::private::AwaitableSubmachine for Button {
    fn state_superstate<'sub>(
        state: &'sub mut <Self as ::sotu::private::Submachine>::State,
    ) -> ::core::option::Option<<Self as ::sotu::private::Submachine>::Superstate<'sub>>
    where
        Self: 'sub,
    {
        <<Button as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
            Button,
        >>::superstate(state)
    }
    fn state_same(
        first: &<Self as ::sotu::private::Submachine>::State,
        second: &<Self as ::sotu::private::Submachine>::State,
    ) -> bool {
        <<Button as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
            Button,
        >>::same_state(first, second)
    }
    fn superstate_same<'first, 'second>(
        first: &<Self as ::sotu::private::Submachine>::Superstate<'first>,
        second: &<Self as ::sotu::private::Submachine>::Superstate<'second>,
    ) -> bool
    where
        Self: 'first + 'second,
    {
        <<Button as ::sotu::awaitable::IntoStateMachine>::Superstate<
            '_,
        > as ::sotu::awaitable::Superstate<Button>>::same_state(first, second)
    }
    fn state_depth(state: &mut <Self as ::sotu::private::Submachine>::State) -> usize {
        <<Button as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::StateExt<
            Button,
        >>::depth(state)
    }
    fn state_is_addressed(
        state: &<Self as ::sotu::private::Submachine>::State,
        event: <Self as ::sotu::private::Submachine>::Event,
    ) -> (bool, <Self as ::sotu::private::Submachine>::Event) {
        <<Button as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
            Button,
        >>::__is_addressed(state, event)
    }
    fn superstate_is_addressed<'sub>(
        superstate: &<Self as ::sotu::private::Submachine>::Superstate<'sub>,
        event: <Self as ::sotu::private::Submachine>::Event,
    ) -> (bool, <Self as ::sotu::private::Submachine>::Event)
    where
        Self: 'sub,
    {
        <<Button as ::sotu::awaitable::IntoStateMachine>::Superstate<
            '_,
        > as ::sotu::awaitable::Superstate<Button>>::__is_addressed(superstate, event)
    }
    fn superstate_depth<'sub>(
        _superstate: &<Self as ::sotu::private::Submachine>::Superstate<'sub>,
    ) -> usize
    where
        Self: 'sub,
    {
        0
    }
    fn superstate_parent<'sub>(
        superstate: <Self as ::sotu::private::Submachine>::Superstate<'sub>,
    ) -> ::core::option::Option<<Self as ::sotu::private::Submachine>::Superstate<'sub>>
    where
        Self: 'sub,
    {
        <<Button as ::sotu::awaitable::IntoStateMachine>::Superstate<
            '_,
        > as ::sotu::awaitable::Superstate<Button>>::superstate(superstate)
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
        <<Button as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
            Button,
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
        <<Button as ::sotu::awaitable::IntoStateMachine>::Superstate<
            '_,
        > as ::sotu::awaitable::Superstate<
            Button,
        >>::__dispatch(superstate, shared_storage, event, context)
    }
    fn enter_state<'ctx, 'data: 'ctx>(
        state: &mut <Self as ::sotu::private::Submachine>::State,
        shared_storage: &mut Self,
        context: &mut <Self as ::sotu::private::Submachine>::Context<'ctx, 'data>,
    ) -> impl ::core::future::Future<Output = ()> {
        <<Button as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
            Button,
        >>::call_entry_action(state, shared_storage, context)
    }
    fn exit_state<'ctx, 'data: 'ctx>(
        state: &mut <Self as ::sotu::private::Submachine>::State,
        shared_storage: &mut Self,
        context: &mut <Self as ::sotu::private::Submachine>::Context<'ctx, 'data>,
    ) -> impl ::core::future::Future<Output = ()> {
        <<Button as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
            Button,
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
        <<Button as ::sotu::awaitable::IntoStateMachine>::Superstate<
            '_,
        > as ::sotu::awaitable::Superstate<
            Button,
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
        <<Button as ::sotu::awaitable::IntoStateMachine>::Superstate<
            '_,
        > as ::sotu::awaitable::Superstate<
            Button,
        >>::call_exit_action(superstate, shared_storage, context)
    }
}
fn main() {
    let _machine = Button::default().state_machine();
}
