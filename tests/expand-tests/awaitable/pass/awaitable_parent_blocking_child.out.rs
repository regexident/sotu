use sotu::prelude::*;
mod door {
    use sotu::prelude::*;
    pub struct Push;
    pub struct Close;
    pub enum Event {
        Push(Push),
        Close(Close),
    }
    impl ::core::convert::From<Push> for Event {
        fn from(value: Push) -> Self {
            Self::Push(value)
        }
    }
    impl ::sotu::event::Variant<Push> for Event {
        fn from_variant(value: Push) -> Self {
            Self::Push(value)
        }
        fn into_variant(self) -> ::core::result::Result<Push, Self> {
            match self {
                Self::Push(value) => ::core::result::Result::Ok(value),
                other => ::core::result::Result::Err(other),
            }
        }
        fn as_variant_ref(&self) -> ::core::option::Option<&Push> {
            match self {
                Self::Push(value) => ::core::option::Option::Some(value),
                _ => ::core::option::Option::None,
            }
        }
    }
    impl ::core::convert::From<Close> for Event {
        fn from(value: Close) -> Self {
            Self::Close(value)
        }
    }
    impl ::sotu::event::Variant<Close> for Event {
        fn from_variant(value: Close) -> Self {
            Self::Close(value)
        }
        fn into_variant(self) -> ::core::result::Result<Close, Self> {
            match self {
                Self::Close(value) => ::core::result::Result::Ok(value),
                other => ::core::result::Result::Err(other),
            }
        }
        fn as_variant_ref(&self) -> ::core::option::Option<&Close> {
            match self {
                Self::Close(value) => ::core::option::Option::Some(value),
                _ => ::core::option::Option::None,
            }
        }
    }
    impl<SotuParentEvent> ::sotu::event::SubEvent<SotuParentEvent> for Event
    where
        SotuParentEvent: ::sotu::event::Variant<Push>,
        SotuParentEvent: ::sotu::event::Variant<Close>,
    {
        fn accepts(parent: &SotuParentEvent) -> bool {
            <SotuParentEvent as ::sotu::event::Variant<Push>>::as_variant_ref(parent)
                .is_some()
                || <SotuParentEvent as ::sotu::event::Variant<
                    Close,
                >>::as_variant_ref(parent)
                    .is_some()
        }
        fn project(
            parent: SotuParentEvent,
        ) -> ::core::result::Result<Self, SotuParentEvent> {
            let parent = match <SotuParentEvent as ::sotu::event::Variant<
                Push,
            >>::into_variant(parent) {
                ::core::result::Result::Ok(value) => {
                    return ::core::result::Result::Ok(Self::Push(value));
                }
                ::core::result::Result::Err(parent) => parent,
            };
            let parent = match <SotuParentEvent as ::sotu::event::Variant<
                Close,
            >>::into_variant(parent) {
                ::core::result::Result::Ok(value) => {
                    return ::core::result::Result::Ok(Self::Close(value));
                }
                ::core::result::Result::Err(parent) => parent,
            };
            ::core::result::Result::Err(parent)
        }
        fn embed(self) -> SotuParentEvent {
            match self {
                Self::Push(value) => {
                    <SotuParentEvent as ::sotu::event::Variant<
                        Push,
                    >>::from_variant(value)
                }
                Self::Close(value) => {
                    <SotuParentEvent as ::sotu::event::Variant<
                        Close,
                    >>::from_variant(value)
                }
            }
        }
    }
    pub enum Lock {
        Engaged,
    }
    pub struct Door {
        pub pushes: u32,
    }
    #[automatically_derived]
    impl ::core::default::Default for Door {
        #[inline]
        fn default() -> Door {
            Door {
                pushes: ::core::default::Default::default(),
            }
        }
    }
    impl Door {
        fn opened(&mut self, event: &Event) -> Outcome<State, Lock> {
            match event {
                Event::Push(_) => {
                    self.pushes += 1;
                    Super
                }
                Event::Close(_) => Exit(Lock::Engaged),
            }
        }
        fn frame() -> Outcome<State, Lock> {
            Handled
        }
    }
    pub enum State {
        Opened {},
    }
    impl State {
        pub fn opened() -> Self {
            Self::Opened {}
        }
    }
    pub enum Superstate {
        Frame {},
    }
    impl ::sotu::blocking::IntoStateMachine for Door {
        type Event = Event;
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = Superstate where Self: 'sub;
        type Exit = Lock;
        fn initial() -> State {
            State::opened()
        }
    }
    impl ::sotu::blocking::State<Door> for State {
        fn call_handler<'ctx, 'data: 'ctx>(
            &mut self,
            shared_storage: &mut Door,
            event: &Event,
            _context: &mut (),
        ) -> ::sotu::Outcome<Self, Lock> {
            match self {
                State::Opened {} => Door::opened(shared_storage, event),
            }
        }
        fn superstate(&mut self) -> ::core::option::Option<Superstate> {
            match self {
                State::Opened {} => ::core::option::Option::Some(Superstate::Frame {}),
            }
        }
    }
    impl ::sotu::blocking::Superstate<Door> for Superstate {
        fn call_handler<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Door,
            _event: &Event,
            _context: &mut (),
        ) -> ::sotu::Outcome<State, Lock> {
            match self {
                Superstate::Frame {} => Door::frame(),
            }
        }
        fn superstate(&mut self) -> ::core::option::Option<Superstate> {
            match self {
                Superstate::Frame {} => ::core::option::Option::None,
            }
        }
        fn same_state(&self, other: &Superstate) -> bool {
            match (self, other) {
                (Superstate::Frame { .. }, Superstate::Frame { .. }) => true,
            }
        }
    }
    impl ::sotu::private::Submachine for Door {
        type Event = Event;
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = Superstate where Self: 'sub;
        type Exit = Lock;
        fn initial() -> State {
            State::opened()
        }
    }
    impl ::sotu::private::BlockingSubmachine for Door {}
    impl ::sotu::private::AwaitableSubmachine for Door {
        fn state_superstate<'sub>(
            state: &'sub mut <Self as ::sotu::private::Submachine>::State,
        ) -> ::core::option::Option<
            <Self as ::sotu::private::Submachine>::Superstate<'sub>,
        >
        where
            Self: 'sub,
        {
            <<Door as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                Door,
            >>::superstate(state)
        }
        fn state_same(
            first: &<Self as ::sotu::private::Submachine>::State,
            second: &<Self as ::sotu::private::Submachine>::State,
        ) -> bool {
            <<Door as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                Door,
            >>::same_state(first, second)
        }
        fn superstate_same<'first, 'second>(
            first: &<Self as ::sotu::private::Submachine>::Superstate<'first>,
            second: &<Self as ::sotu::private::Submachine>::Superstate<'second>,
        ) -> bool
        where
            Self: 'first + 'second,
        {
            <<Door as ::sotu::blocking::IntoStateMachine>::Superstate<
                '_,
            > as ::sotu::blocking::Superstate<Door>>::same_state(first, second)
        }
        fn state_depth(
            state: &mut <Self as ::sotu::private::Submachine>::State,
        ) -> usize {
            <<Door as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::StateExt<
                Door,
            >>::depth(state)
        }
        fn state_is_addressed(
            state: &<Self as ::sotu::private::Submachine>::State,
            event: <Self as ::sotu::private::Submachine>::Event,
        ) -> (bool, <Self as ::sotu::private::Submachine>::Event) {
            <<Door as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                Door,
            >>::__is_addressed(state, event)
        }
        fn superstate_is_addressed<'sub>(
            superstate: &<Self as ::sotu::private::Submachine>::Superstate<'sub>,
            event: <Self as ::sotu::private::Submachine>::Event,
        ) -> (bool, <Self as ::sotu::private::Submachine>::Event)
        where
            Self: 'sub,
        {
            <<Door as ::sotu::blocking::IntoStateMachine>::Superstate<
                '_,
            > as ::sotu::blocking::Superstate<Door>>::__is_addressed(superstate, event)
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
        ) -> ::core::option::Option<
            <Self as ::sotu::private::Submachine>::Superstate<'sub>,
        >
        where
            Self: 'sub,
        {
            match superstate {
                Superstate::Frame {} => ::core::option::Option::None,
            }
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
            ::core::future::ready(
                <<Door as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                    Door,
                >>::__dispatch(state, shared_storage, event, context),
            )
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
            ::core::future::ready(
                <<Door as ::sotu::blocking::IntoStateMachine>::Superstate<
                    '_,
                > as ::sotu::blocking::Superstate<
                    Door,
                >>::__dispatch(superstate, shared_storage, event, context),
            )
        }
        fn enter_state<'ctx, 'data: 'ctx>(
            state: &mut <Self as ::sotu::private::Submachine>::State,
            shared_storage: &mut Self,
            context: &mut <Self as ::sotu::private::Submachine>::Context<'ctx, 'data>,
        ) -> impl ::core::future::Future<Output = ()> {
            ::core::future::ready(
                <<Door as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                    Door,
                >>::call_entry_action(state, shared_storage, context),
            )
        }
        fn exit_state<'ctx, 'data: 'ctx>(
            state: &mut <Self as ::sotu::private::Submachine>::State,
            shared_storage: &mut Self,
            context: &mut <Self as ::sotu::private::Submachine>::Context<'ctx, 'data>,
        ) -> impl ::core::future::Future<Output = ()> {
            ::core::future::ready(
                <<Door as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                    Door,
                >>::call_exit_action(state, shared_storage, context),
            )
        }
        fn enter_superstate<'ctx, 'data: 'ctx, 'sub>(
            superstate: &mut <Self as ::sotu::private::Submachine>::Superstate<'sub>,
            shared_storage: &mut Self,
            context: &mut <Self as ::sotu::private::Submachine>::Context<'ctx, 'data>,
        ) -> impl ::core::future::Future<Output = ()>
        where
            Self: 'sub,
        {
            ::core::future::ready(
                <<Door as ::sotu::blocking::IntoStateMachine>::Superstate<
                    '_,
                > as ::sotu::blocking::Superstate<
                    Door,
                >>::call_entry_action(superstate, shared_storage, context),
            )
        }
        fn exit_superstate<'ctx, 'data: 'ctx, 'sub>(
            superstate: &mut <Self as ::sotu::private::Submachine>::Superstate<'sub>,
            shared_storage: &mut Self,
            context: &mut <Self as ::sotu::private::Submachine>::Context<'ctx, 'data>,
        ) -> impl ::core::future::Future<Output = ()>
        where
            Self: 'sub,
        {
            ::core::future::ready(
                <<Door as ::sotu::blocking::IntoStateMachine>::Superstate<
                    '_,
                > as ::sotu::blocking::Superstate<
                    Door,
                >>::call_exit_action(superstate, shared_storage, context),
            )
        }
        fn before_dispatch<'ctx, 'data: 'ctx, 'sub>(
            shared_storage: &mut Self,
            node: ::sotu::StateOrSuperstate<
                '_,
                <Self as ::sotu::private::Submachine>::State,
                <Self as ::sotu::private::Submachine>::Superstate<'sub>,
            >,
            event: &<Self as ::sotu::private::Submachine>::Event,
            context: &mut <Self as ::sotu::private::Submachine>::Context<'ctx, 'data>,
        ) -> impl ::core::future::Future<Output = ()>
        where
            Self: 'sub,
        {
            ::core::future::ready(
                <Door as ::sotu::blocking::IntoStateMachine>::before_dispatch(
                    shared_storage,
                    node,
                    event,
                    context,
                ),
            )
        }
        fn after_dispatch<'ctx, 'data: 'ctx, 'sub>(
            shared_storage: &mut Self,
            node: ::sotu::StateOrSuperstate<
                '_,
                <Self as ::sotu::private::Submachine>::State,
                <Self as ::sotu::private::Submachine>::Superstate<'sub>,
            >,
            event: &<Self as ::sotu::private::Submachine>::Event,
            context: &mut <Self as ::sotu::private::Submachine>::Context<'ctx, 'data>,
        ) -> impl ::core::future::Future<Output = ()>
        where
            Self: 'sub,
        {
            ::core::future::ready(
                <Door as ::sotu::blocking::IntoStateMachine>::after_dispatch(
                    shared_storage,
                    node,
                    event,
                    context,
                ),
            )
        }
        fn before_transition<'ctx, 'data: 'ctx>(
            shared_storage: &mut Self,
            source: &<Self as ::sotu::private::Submachine>::State,
            target: &<Self as ::sotu::private::Submachine>::State,
            context: &mut <Self as ::sotu::private::Submachine>::Context<'ctx, 'data>,
        ) -> impl ::core::future::Future<Output = ()> {
            ::core::future::ready(
                <Door as ::sotu::blocking::IntoStateMachine>::before_transition(
                    shared_storage,
                    source,
                    target,
                    context,
                ),
            )
        }
        fn after_transition<'ctx, 'data: 'ctx>(
            shared_storage: &mut Self,
            source: &<Self as ::sotu::private::Submachine>::State,
            target: &<Self as ::sotu::private::Submachine>::State,
            context: &mut <Self as ::sotu::private::Submachine>::Context<'ctx, 'data>,
        ) -> impl ::core::future::Future<Output = ()> {
            ::core::future::ready(
                <Door as ::sotu::blocking::IntoStateMachine>::after_transition(
                    shared_storage,
                    source,
                    target,
                    context,
                ),
            )
        }
        fn internal_transition_nesting(
            source: &mut <Self as ::sotu::private::Submachine>::State,
            target: &mut <Self as ::sotu::private::Submachine>::State,
            exit_levels: usize,
        ) -> usize {
            <<Door as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                Door,
            >>::__internal_transition_nesting(source, target, exit_levels)
        }
        fn before_internal_transition<'ctx, 'data: 'ctx>(
            source: &mut <Self as ::sotu::private::Submachine>::State,
            target: &mut <Self as ::sotu::private::Submachine>::State,
            exit_levels: usize,
            nesting: usize,
            shared_storage: &mut Self,
            context: &mut <Self as ::sotu::private::Submachine>::Context<'ctx, 'data>,
        ) -> impl ::core::future::Future<Output = ()> {
            ::core::future::ready(
                <<Door as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                    Door,
                >>::__before_internal_transition(
                    source,
                    target,
                    exit_levels,
                    nesting,
                    shared_storage,
                    context,
                ),
            )
        }
        fn after_internal_transition<'ctx, 'data: 'ctx>(
            source: &mut <Self as ::sotu::private::Submachine>::State,
            target: &mut <Self as ::sotu::private::Submachine>::State,
            exit_levels: usize,
            nesting: usize,
            shared_storage: &mut Self,
            context: &mut <Self as ::sotu::private::Submachine>::Context<'ctx, 'data>,
        ) -> impl ::core::future::Future<Output = ()> {
            ::core::future::ready(
                <<Door as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                    Door,
                >>::__after_internal_transition(
                    source,
                    target,
                    exit_levels,
                    nesting,
                    shared_storage,
                    context,
                ),
            )
        }
    }
    impl Superstate {
        fn __depth(&self) -> usize {
            match self {
                Superstate::Frame { .. } => 1usize,
            }
        }
    }
}
pub struct Start;
pub enum Event {
    Start(Start),
    Push(door::Push),
    Close(door::Close),
}
impl ::core::convert::From<Start> for Event {
    fn from(value: Start) -> Self {
        Self::Start(value)
    }
}
impl ::sotu::event::Variant<Start> for Event {
    fn from_variant(value: Start) -> Self {
        Self::Start(value)
    }
    fn into_variant(self) -> ::core::result::Result<Start, Self> {
        match self {
            Self::Start(value) => ::core::result::Result::Ok(value),
            other => ::core::result::Result::Err(other),
        }
    }
    fn as_variant_ref(&self) -> ::core::option::Option<&Start> {
        match self {
            Self::Start(value) => ::core::option::Option::Some(value),
            _ => ::core::option::Option::None,
        }
    }
}
impl ::core::convert::From<door::Push> for Event {
    fn from(value: door::Push) -> Self {
        Self::Push(value)
    }
}
impl ::sotu::event::Variant<door::Push> for Event {
    fn from_variant(value: door::Push) -> Self {
        Self::Push(value)
    }
    fn into_variant(self) -> ::core::result::Result<door::Push, Self> {
        match self {
            Self::Push(value) => ::core::result::Result::Ok(value),
            other => ::core::result::Result::Err(other),
        }
    }
    fn as_variant_ref(&self) -> ::core::option::Option<&door::Push> {
        match self {
            Self::Push(value) => ::core::option::Option::Some(value),
            _ => ::core::option::Option::None,
        }
    }
}
impl ::core::convert::From<door::Close> for Event {
    fn from(value: door::Close) -> Self {
        Self::Close(value)
    }
}
impl ::sotu::event::Variant<door::Close> for Event {
    fn from_variant(value: door::Close) -> Self {
        Self::Close(value)
    }
    fn into_variant(self) -> ::core::result::Result<door::Close, Self> {
        match self {
            Self::Close(value) => ::core::result::Result::Ok(value),
            other => ::core::result::Result::Err(other),
        }
    }
    fn as_variant_ref(&self) -> ::core::option::Option<&door::Close> {
        match self {
            Self::Close(value) => ::core::option::Option::Some(value),
            _ => ::core::option::Option::None,
        }
    }
}
impl<SotuParentEvent> ::sotu::event::SubEvent<SotuParentEvent> for Event
where
    SotuParentEvent: ::sotu::event::Variant<Start>,
    SotuParentEvent: ::sotu::event::Variant<door::Push>,
    SotuParentEvent: ::sotu::event::Variant<door::Close>,
{
    fn accepts(parent: &SotuParentEvent) -> bool {
        <SotuParentEvent as ::sotu::event::Variant<Start>>::as_variant_ref(parent)
            .is_some()
            || <SotuParentEvent as ::sotu::event::Variant<
                door::Push,
            >>::as_variant_ref(parent)
                .is_some()
            || <SotuParentEvent as ::sotu::event::Variant<
                door::Close,
            >>::as_variant_ref(parent)
                .is_some()
    }
    fn project(
        parent: SotuParentEvent,
    ) -> ::core::result::Result<Self, SotuParentEvent> {
        let parent = match <SotuParentEvent as ::sotu::event::Variant<
            Start,
        >>::into_variant(parent) {
            ::core::result::Result::Ok(value) => {
                return ::core::result::Result::Ok(Self::Start(value));
            }
            ::core::result::Result::Err(parent) => parent,
        };
        let parent = match <SotuParentEvent as ::sotu::event::Variant<
            door::Push,
        >>::into_variant(parent) {
            ::core::result::Result::Ok(value) => {
                return ::core::result::Result::Ok(Self::Push(value));
            }
            ::core::result::Result::Err(parent) => parent,
        };
        let parent = match <SotuParentEvent as ::sotu::event::Variant<
            door::Close,
        >>::into_variant(parent) {
            ::core::result::Result::Ok(value) => {
                return ::core::result::Result::Ok(Self::Close(value));
            }
            ::core::result::Result::Err(parent) => parent,
        };
        ::core::result::Result::Err(parent)
    }
    fn embed(self) -> SotuParentEvent {
        match self {
            Self::Start(value) => {
                <SotuParentEvent as ::sotu::event::Variant<Start>>::from_variant(value)
            }
            Self::Push(value) => {
                <SotuParentEvent as ::sotu::event::Variant<
                    door::Push,
                >>::from_variant(value)
            }
            Self::Close(value) => {
                <SotuParentEvent as ::sotu::event::Variant<
                    door::Close,
                >>::from_variant(value)
            }
        }
    }
}
pub struct Hall {
    pub door: Option<door::Door>,
    pub entries: u32,
}
#[automatically_derived]
impl ::core::default::Default for Hall {
    #[inline]
    fn default() -> Hall {
        Hall {
            door: ::core::default::Default::default(),
            entries: ::core::default::Default::default(),
        }
    }
}
impl Hall {
    fn idle(event: &Event) -> Outcome<State> {
        match event {
            Event::Start(_) => Transition(Superstate::loading()),
            _ => Super,
        }
    }
    fn loading(submachine: &mut door::Door, event: &Event) -> Outcome<State> {
        match event {
            Event::Push(_) if submachine.pushes > 3 => Transition(State::idle()),
            _ => Handled,
        }
    }
    async fn door_exited(&mut self, exit: door::Lock) -> Outcome<State> {
        let door::Lock::Engaged = exit;
        self.entries += 1;
        Transition(State::idle())
    }
    async fn enter_loading(&mut self) {
        self.entries += 1;
    }
    fn exit_loading(submachine: &mut door::Door) {
        submachine.pushes = 0;
    }
}
pub enum State {
    Idle {},
    Loading(<door::Door as ::sotu::private::Submachine>::State),
}
impl State {
    pub fn idle() -> Self {
        Self::Idle {}
    }
    pub fn loading(state: <door::Door as ::sotu::private::Submachine>::State) -> Self {
        Self::Loading(state)
    }
}
pub enum Superstate<'sub> {
    Loading(
        ::sotu::SubmachineNode<
            <door::Door as ::sotu::private::Submachine>::Superstate<'sub>,
        >,
    ),
}
impl Superstate<'_> {
    pub fn loading() -> State {
        State::Loading(<door::Door as ::sotu::private::Submachine>::initial())
    }
}
impl ::sotu::awaitable::IntoStateMachine for Hall {
    type Event = Event;
    type Context<'ctx, 'data: 'ctx> = u32;
    type State = State;
    type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::idle()
    }
}
impl ::sotu::awaitable::State<Hall> for State
where
    door::Door: ::sotu::private::AwaitableSubmachine,
{
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Hall,
        event: &Event,
        _context: &mut u32,
    ) -> impl ::core::future::Future<
        Output = ::sotu::Outcome<Self, ::core::convert::Infallible>,
    > {
        async move {
            match self {
                State::Idle {} => Hall::idle(event),
                State::Loading(_) => ::sotu::Outcome::Super,
            }
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Hall,
        context: &mut u32,
    ) -> impl ::core::future::Future<Output = ()> {
        async move {
            match self {
                State::Idle { .. } => {}
                State::Loading(state) => {
                    if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                        door::Door,
                    >::get_mut(&mut shared_storage.door) {
                        {
                            let mut child_context = (sotu::event::unit_context)(context);
                            <door::Door as ::sotu::private::AwaitableSubmachine>::enter_state(
                                    state,
                                    submachine,
                                    &mut child_context,
                                )
                                .await
                        };
                    }
                }
            }
        }
    }
    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Hall,
        context: &mut u32,
    ) -> impl ::core::future::Future<Output = ()> {
        async move {
            match self {
                State::Idle { .. } => {}
                State::Loading(state) => {
                    if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                        door::Door,
                    >::get_mut(&mut shared_storage.door) {
                        {
                            let mut child_context = (sotu::event::unit_context)(context);
                            <door::Door as ::sotu::private::AwaitableSubmachine>::exit_state(
                                    state,
                                    submachine,
                                    &mut child_context,
                                )
                                .await
                        };
                    }
                }
            }
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate<'_>> {
        match self {
            State::Idle {} => ::core::option::Option::None,
            State::Loading(state) => {
                ::core::option::Option::Some(
                    Superstate::Loading(
                        match <door::Door as ::sotu::private::AwaitableSubmachine>::state_superstate(
                            state,
                        ) {
                            ::core::option::Option::Some(superstate) => {
                                ::sotu::SubmachineNode::Inner(superstate)
                            }
                            ::core::option::Option::None => ::sotu::SubmachineNode::Mount,
                        },
                    ),
                )
            }
        }
    }
    fn same_state(&self, other: &Self) -> bool {
        match (self, other) {
            (State::Idle { .. }, State::Idle { .. }) => true,
            (State::Loading(first), State::Loading(second)) => {
                <door::Door as ::sotu::private::AwaitableSubmachine>::state_same(
                    first,
                    second,
                )
            }
            _ => false,
        }
    }
    fn __is_addressed(&self, event: Event) -> (bool, Event) {
        match self {
            State::Loading(node) => {
                match Hall::__project_into_loading(
                    event,
                    |event| {
                        <door::Door as ::sotu::private::AwaitableSubmachine>::state_is_addressed(
                            node,
                            event,
                        )
                    },
                ) {
                    ::core::result::Result::Ok(addressed) => addressed,
                    ::core::result::Result::Err(event) => (false, event),
                }
            }
            _ => (true, event),
        }
    }
    #[allow(unreachable_code)]
    fn __dispatch<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Hall,
        event: Event,
        context: &mut u32,
    ) -> impl ::core::future::Future<
        Output = ::sotu::private::Dispatched<Self, ::core::convert::Infallible, Event>,
    > {
        async move {
            match self {
                State::Loading(node) => {
                    let mount_offset = <door::Door as ::sotu::private::AwaitableSubmachine>::state_depth(
                        node,
                    );
                    let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                        door::Door,
                    >::get_mut(&mut shared_storage.door) else {
                        return ::sotu::private::Dispatched {
                            outcome: ::sotu::private::NodeOutcome::DeferToMount {
                                mount_offset,
                            },
                            event: event,
                        };
                    };
                    let visited = Hall::__project_awaited_into_loading(
                            event,
                            async |event| {
                                {
                                    let mut child_context = (sotu::event::unit_context)(
                                        context,
                                    );
                                    <door::Door as ::sotu::private::AwaitableSubmachine>::before_dispatch(
                                            submachine,
                                            ::sotu::StateOrSuperstate::State(&*node),
                                            &event,
                                            &mut child_context,
                                        )
                                        .await
                                };
                                let ::sotu::private::Dispatched { outcome, event } = {
                                    let mut child_context = (sotu::event::unit_context)(
                                        context,
                                    );
                                    <door::Door as ::sotu::private::AwaitableSubmachine>::dispatch_state(
                                            node,
                                            submachine,
                                            event,
                                            &mut child_context,
                                        )
                                        .await
                                };
                                {
                                    let mut child_context = (sotu::event::unit_context)(
                                        context,
                                    );
                                    <door::Door as ::sotu::private::AwaitableSubmachine>::after_dispatch(
                                            submachine,
                                            ::sotu::StateOrSuperstate::State(&*node),
                                            &event,
                                            &mut child_context,
                                        )
                                        .await
                                };
                                (outcome, event)
                            },
                        )
                        .await;
                    match visited {
                        ::core::result::Result::Ok((outcome, event)) => {
                            ::sotu::private::Dispatched {
                                outcome: match outcome {
                                    ::sotu::private::NodeOutcome::Handled => {
                                        ::sotu::private::NodeOutcome::Handled
                                    }
                                    ::sotu::private::NodeOutcome::Super => {
                                        ::sotu::private::NodeOutcome::Super
                                    }
                                    ::sotu::private::NodeOutcome::Transition(target) => {
                                        ::sotu::private::NodeOutcome::Transition(
                                            State::Loading(target),
                                        )
                                    }
                                    ::sotu::private::NodeOutcome::ExternalTransition {
                                        target,
                                        handler_offset,
                                    } => {
                                        ::sotu::private::NodeOutcome::ExternalTransition {
                                            target: State::Loading(target),
                                            handler_offset,
                                        }
                                    }
                                    ::sotu::private::NodeOutcome::Exit(exit) => {
                                        match Hall::door_exited(shared_storage, exit).await {
                                            ::sotu::Outcome::Handled => {
                                                ::sotu::private::NodeOutcome::Handled
                                            }
                                            ::sotu::Outcome::Super => {
                                                ::sotu::private::NodeOutcome::DeferToMount {
                                                    mount_offset,
                                                }
                                            }
                                            ::sotu::Outcome::Transition(target) => {
                                                ::sotu::private::NodeOutcome::Transition(target)
                                            }
                                            ::sotu::Outcome::ExternalTransition(target) => {
                                                ::sotu::private::NodeOutcome::ExternalTransition {
                                                    target,
                                                    handler_offset: mount_offset,
                                                }
                                            }
                                            ::sotu::Outcome::Exit(parent_exit) => {
                                                ::sotu::private::NodeOutcome::Exit(parent_exit)
                                            }
                                        }
                                    }
                                    ::sotu::private::NodeOutcome::DeferToMount {
                                        mount_offset,
                                    } => {
                                        ::sotu::private::NodeOutcome::DeferToMount {
                                            mount_offset,
                                        }
                                    }
                                },
                                event: event,
                            }
                        }
                        ::core::result::Result::Err(event) => {
                            ::sotu::private::Dispatched {
                                outcome: ::sotu::private::NodeOutcome::DeferToMount {
                                    mount_offset,
                                },
                                event: event,
                            }
                        }
                    }
                }
                _ => {
                    let outcome = ::sotu::awaitable::State::<
                        Hall,
                    >::call_handler(self, shared_storage, &event, context)
                        .await;
                    ::sotu::private::Dispatched {
                        outcome: ::sotu::private::NodeOutcome::from(outcome),
                        event: event,
                    }
                }
            }
        }
    }
    fn __internal_transition_nesting(
        &mut self,
        target: &mut Self,
        exit_levels: usize,
    ) -> usize {
        match (self, target) {
            (State::Loading(source), State::Loading(target)) => {
                if exit_levels
                    <= <door::Door as ::sotu::private::AwaitableSubmachine>::state_depth(
                        source,
                    )
                {
                    1
                        + <door::Door as ::sotu::private::AwaitableSubmachine>::internal_transition_nesting(
                            source,
                            target,
                            exit_levels,
                        )
                } else {
                    0
                }
            }
            _ => 0,
        }
    }
    fn __before_internal_transition<'ctx, 'data: 'ctx>(
        &mut self,
        target: &mut Self,
        exit_levels: usize,
        nesting: usize,
        shared_storage: &mut Hall,
        context: &mut u32,
    ) -> impl ::core::future::Future<Output = ()> {
        async move {
            match (self, target) {
                (State::Loading(source), State::Loading(target)) => {
                    if exit_levels
                        > <door::Door as ::sotu::private::AwaitableSubmachine>::state_depth(
                            source,
                        )
                    {
                        return;
                    }
                    let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                        door::Door,
                    >::get_mut(&mut shared_storage.door) else {
                        return;
                    };
                    match nesting.checked_sub(1) {
                        ::core::option::Option::Some(0) => {
                            {
                                let mut child_context = (sotu::event::unit_context)(
                                    context,
                                );
                                <door::Door as ::sotu::private::AwaitableSubmachine>::before_transition(
                                        submachine,
                                        source,
                                        target,
                                        &mut child_context,
                                    )
                                    .await
                            };
                        }
                        ::core::option::Option::Some(deeper) => {
                            {
                                let mut child_context = (sotu::event::unit_context)(
                                    context,
                                );
                                <door::Door as ::sotu::private::AwaitableSubmachine>::before_internal_transition(
                                        source,
                                        target,
                                        exit_levels,
                                        deeper,
                                        submachine,
                                        &mut child_context,
                                    )
                                    .await
                            };
                        }
                        ::core::option::Option::None => {}
                    }
                }
                _ => {}
            }
        }
    }
    fn __after_internal_transition<'ctx, 'data: 'ctx>(
        &mut self,
        target: &mut Self,
        exit_levels: usize,
        nesting: usize,
        shared_storage: &mut Hall,
        context: &mut u32,
    ) -> impl ::core::future::Future<Output = ()> {
        async move {
            match (self, target) {
                (State::Loading(source), State::Loading(target)) => {
                    if exit_levels
                        > <door::Door as ::sotu::private::AwaitableSubmachine>::state_depth(
                            source,
                        )
                    {
                        return;
                    }
                    let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                        door::Door,
                    >::get_mut(&mut shared_storage.door) else {
                        return;
                    };
                    match nesting.checked_sub(1) {
                        ::core::option::Option::Some(0) => {
                            {
                                let mut child_context = (sotu::event::unit_context)(
                                    context,
                                );
                                <door::Door as ::sotu::private::AwaitableSubmachine>::after_transition(
                                        submachine,
                                        source,
                                        target,
                                        &mut child_context,
                                    )
                                    .await
                            };
                        }
                        ::core::option::Option::Some(deeper) => {
                            {
                                let mut child_context = (sotu::event::unit_context)(
                                    context,
                                );
                                <door::Door as ::sotu::private::AwaitableSubmachine>::after_internal_transition(
                                        source,
                                        target,
                                        exit_levels,
                                        deeper,
                                        submachine,
                                        &mut child_context,
                                    )
                                    .await
                            };
                        }
                        ::core::option::Option::None => {}
                    }
                }
                _ => {}
            }
        }
    }
}
impl<'sub> ::sotu::awaitable::Superstate<Hall> for Superstate<'sub>
where
    door::Door: ::sotu::private::AwaitableSubmachine,
{
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Hall,
        event: &Event,
        _context: &mut u32,
    ) -> impl ::core::future::Future<
        Output = ::sotu::Outcome<State, ::core::convert::Infallible>,
    > {
        async move {
            match self {
                Superstate::Loading(::sotu::SubmachineNode::Inner(_)) => {
                    ::sotu::Outcome::Super
                }
                Superstate::Loading(::sotu::SubmachineNode::Mount) => {
                    match ::sotu::private::MountField::<
                        door::Door,
                    >::get_mut(&mut shared_storage.door) {
                        ::core::option::Option::Some(submachine) => {
                            Hall::loading(submachine, event)
                        }
                        ::core::option::Option::None => ::sotu::Outcome::Super,
                    }
                }
            }
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Hall,
        context: &mut u32,
    ) -> impl ::core::future::Future<Output = ()> {
        async move {
            match self {
                Superstate::Loading(::sotu::SubmachineNode::Mount) => {
                    ::sotu::private::DefaultInstance::<
                        door::Door,
                    >::fill_default(&mut shared_storage.door);
                    Hall::enter_loading(shared_storage).await;
                }
                Superstate::Loading(::sotu::SubmachineNode::Inner(state)) => {
                    if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                        door::Door,
                    >::get_mut(&mut shared_storage.door) {
                        {
                            let mut child_context = (sotu::event::unit_context)(context);
                            <door::Door as ::sotu::private::AwaitableSubmachine>::enter_superstate(
                                    state,
                                    submachine,
                                    &mut child_context,
                                )
                                .await
                        };
                    }
                }
            }
        }
    }
    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Hall,
        context: &mut u32,
    ) -> impl ::core::future::Future<Output = ()> {
        async move {
            match self {
                Superstate::Loading(::sotu::SubmachineNode::Mount) => {
                    match ::sotu::private::MountField::<
                        door::Door,
                    >::get_mut(&mut shared_storage.door) {
                        ::core::option::Option::Some(submachine) => {
                            Hall::exit_loading(submachine)
                        }
                        ::core::option::Option::None => {}
                    };
                    ::sotu::private::MountField::<
                        door::Door,
                    >::release(&mut shared_storage.door);
                }
                Superstate::Loading(::sotu::SubmachineNode::Inner(state)) => {
                    if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                        door::Door,
                    >::get_mut(&mut shared_storage.door) {
                        {
                            let mut child_context = (sotu::event::unit_context)(context);
                            <door::Door as ::sotu::private::AwaitableSubmachine>::exit_superstate(
                                    state,
                                    submachine,
                                    &mut child_context,
                                )
                                .await
                        };
                    }
                }
            }
        }
    }
    fn superstate(self) -> ::core::option::Option<Self> {
        match self {
            Superstate::Loading(::sotu::SubmachineNode::Mount) => {
                ::core::option::Option::None
            }
            Superstate::Loading(::sotu::SubmachineNode::Inner(superstate)) => {
                ::core::option::Option::Some(
                    Superstate::Loading(
                        match <door::Door as ::sotu::private::AwaitableSubmachine>::superstate_parent(
                            superstate,
                        ) {
                            ::core::option::Option::Some(parent) => {
                                ::sotu::SubmachineNode::Inner(parent)
                            }
                            ::core::option::Option::None => ::sotu::SubmachineNode::Mount,
                        },
                    ),
                )
            }
        }
    }
    fn same_state(&self, other: &Superstate<'_>) -> bool {
        match (self, other) {
            (
                Superstate::Loading(::sotu::SubmachineNode::Mount),
                Superstate::Loading(::sotu::SubmachineNode::Mount),
            ) => true,
            (
                Superstate::Loading(::sotu::SubmachineNode::Inner(first)),
                Superstate::Loading(::sotu::SubmachineNode::Inner(second)),
            ) => {
                <door::Door as ::sotu::private::AwaitableSubmachine>::superstate_same(
                    first,
                    second,
                )
            }
            _ => false,
        }
    }
    fn __is_addressed(&self, event: Event) -> (bool, Event) {
        match self {
            Superstate::Loading(::sotu::SubmachineNode::Inner(node)) => {
                match Hall::__project_into_loading(
                    event,
                    |event| {
                        <door::Door as ::sotu::private::AwaitableSubmachine>::superstate_is_addressed(
                            node,
                            event,
                        )
                    },
                ) {
                    ::core::result::Result::Ok(addressed) => addressed,
                    ::core::result::Result::Err(event) => (false, event),
                }
            }
            _ => (true, event),
        }
    }
    #[allow(unreachable_code)]
    fn __dispatch<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Hall,
        event: Event,
        context: &mut u32,
    ) -> impl ::core::future::Future<
        Output = ::sotu::private::Dispatched<State, ::core::convert::Infallible, Event>,
    > {
        async move {
            match self {
                Superstate::Loading(::sotu::SubmachineNode::Inner(node)) => {
                    let mount_offset = <door::Door as ::sotu::private::AwaitableSubmachine>::superstate_depth(
                        node,
                    );
                    let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                        door::Door,
                    >::get_mut(&mut shared_storage.door) else {
                        return ::sotu::private::Dispatched {
                            outcome: ::sotu::private::NodeOutcome::DeferToMount {
                                mount_offset,
                            },
                            event: event,
                        };
                    };
                    let visited = Hall::__project_awaited_into_loading(
                            event,
                            async |event| {
                                {
                                    let mut child_context = (sotu::event::unit_context)(
                                        context,
                                    );
                                    <door::Door as ::sotu::private::AwaitableSubmachine>::before_dispatch(
                                            submachine,
                                            ::sotu::StateOrSuperstate::Superstate(&*node),
                                            &event,
                                            &mut child_context,
                                        )
                                        .await
                                };
                                let ::sotu::private::Dispatched { outcome, event } = {
                                    let mut child_context = (sotu::event::unit_context)(
                                        context,
                                    );
                                    <door::Door as ::sotu::private::AwaitableSubmachine>::dispatch_superstate(
                                            node,
                                            submachine,
                                            event,
                                            &mut child_context,
                                        )
                                        .await
                                };
                                {
                                    let mut child_context = (sotu::event::unit_context)(
                                        context,
                                    );
                                    <door::Door as ::sotu::private::AwaitableSubmachine>::after_dispatch(
                                            submachine,
                                            ::sotu::StateOrSuperstate::Superstate(&*node),
                                            &event,
                                            &mut child_context,
                                        )
                                        .await
                                };
                                (outcome, event)
                            },
                        )
                        .await;
                    match visited {
                        ::core::result::Result::Ok((outcome, event)) => {
                            ::sotu::private::Dispatched {
                                outcome: match outcome {
                                    ::sotu::private::NodeOutcome::Handled => {
                                        ::sotu::private::NodeOutcome::Handled
                                    }
                                    ::sotu::private::NodeOutcome::Super => {
                                        ::sotu::private::NodeOutcome::Super
                                    }
                                    ::sotu::private::NodeOutcome::Transition(target) => {
                                        ::sotu::private::NodeOutcome::Transition(
                                            State::Loading(target),
                                        )
                                    }
                                    ::sotu::private::NodeOutcome::ExternalTransition {
                                        target,
                                        handler_offset,
                                    } => {
                                        ::sotu::private::NodeOutcome::ExternalTransition {
                                            target: State::Loading(target),
                                            handler_offset,
                                        }
                                    }
                                    ::sotu::private::NodeOutcome::Exit(exit) => {
                                        match Hall::door_exited(shared_storage, exit).await {
                                            ::sotu::Outcome::Handled => {
                                                ::sotu::private::NodeOutcome::Handled
                                            }
                                            ::sotu::Outcome::Super => {
                                                ::sotu::private::NodeOutcome::DeferToMount {
                                                    mount_offset,
                                                }
                                            }
                                            ::sotu::Outcome::Transition(target) => {
                                                ::sotu::private::NodeOutcome::Transition(target)
                                            }
                                            ::sotu::Outcome::ExternalTransition(target) => {
                                                ::sotu::private::NodeOutcome::ExternalTransition {
                                                    target,
                                                    handler_offset: mount_offset,
                                                }
                                            }
                                            ::sotu::Outcome::Exit(parent_exit) => {
                                                ::sotu::private::NodeOutcome::Exit(parent_exit)
                                            }
                                        }
                                    }
                                    ::sotu::private::NodeOutcome::DeferToMount {
                                        mount_offset,
                                    } => {
                                        ::sotu::private::NodeOutcome::DeferToMount {
                                            mount_offset,
                                        }
                                    }
                                },
                                event: event,
                            }
                        }
                        ::core::result::Result::Err(event) => {
                            ::sotu::private::Dispatched {
                                outcome: ::sotu::private::NodeOutcome::DeferToMount {
                                    mount_offset,
                                },
                                event: event,
                            }
                        }
                    }
                }
                _ => {
                    let outcome = ::sotu::awaitable::Superstate::<
                        Hall,
                    >::call_handler(self, shared_storage, &event, context)
                        .await;
                    ::sotu::private::Dispatched {
                        outcome: ::sotu::private::NodeOutcome::from(outcome),
                        event: event,
                    }
                }
            }
        }
    }
}
impl ::sotu::private::Submachine for Hall {
    type Event = Event;
    type Context<'ctx, 'data: 'ctx> = u32;
    type State = State;
    type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::idle()
    }
}
impl ::sotu::private::ExitlessSubmachine for Hall {}
impl ::sotu::private::AwaitableSubmachine for Hall
where
    door::Door: ::sotu::private::AwaitableSubmachine,
{
    fn state_superstate<'sub>(
        state: &'sub mut <Self as ::sotu::private::Submachine>::State,
    ) -> ::core::option::Option<<Self as ::sotu::private::Submachine>::Superstate<'sub>>
    where
        Self: 'sub,
    {
        <<Hall as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
            Hall,
        >>::superstate(state)
    }
    fn state_same(
        first: &<Self as ::sotu::private::Submachine>::State,
        second: &<Self as ::sotu::private::Submachine>::State,
    ) -> bool {
        <<Hall as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
            Hall,
        >>::same_state(first, second)
    }
    fn superstate_same<'first, 'second>(
        first: &<Self as ::sotu::private::Submachine>::Superstate<'first>,
        second: &<Self as ::sotu::private::Submachine>::Superstate<'second>,
    ) -> bool
    where
        Self: 'first + 'second,
    {
        <<Hall as ::sotu::awaitable::IntoStateMachine>::Superstate<
            '_,
        > as ::sotu::awaitable::Superstate<Hall>>::same_state(first, second)
    }
    fn state_depth(state: &mut <Self as ::sotu::private::Submachine>::State) -> usize {
        <<Hall as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::StateExt<
            Hall,
        >>::depth(state)
    }
    fn state_is_addressed(
        state: &<Self as ::sotu::private::Submachine>::State,
        event: <Self as ::sotu::private::Submachine>::Event,
    ) -> (bool, <Self as ::sotu::private::Submachine>::Event) {
        <<Hall as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
            Hall,
        >>::__is_addressed(state, event)
    }
    fn superstate_is_addressed<'sub>(
        superstate: &<Self as ::sotu::private::Submachine>::Superstate<'sub>,
        event: <Self as ::sotu::private::Submachine>::Event,
    ) -> (bool, <Self as ::sotu::private::Submachine>::Event)
    where
        Self: 'sub,
    {
        <<Hall as ::sotu::awaitable::IntoStateMachine>::Superstate<
            '_,
        > as ::sotu::awaitable::Superstate<Hall>>::__is_addressed(superstate, event)
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
        <<Hall as ::sotu::awaitable::IntoStateMachine>::Superstate<
            '_,
        > as ::sotu::awaitable::Superstate<Hall>>::superstate(superstate)
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
        <<Hall as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
            Hall,
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
        <<Hall as ::sotu::awaitable::IntoStateMachine>::Superstate<
            '_,
        > as ::sotu::awaitable::Superstate<
            Hall,
        >>::__dispatch(superstate, shared_storage, event, context)
    }
    fn enter_state<'ctx, 'data: 'ctx>(
        state: &mut <Self as ::sotu::private::Submachine>::State,
        shared_storage: &mut Self,
        context: &mut <Self as ::sotu::private::Submachine>::Context<'ctx, 'data>,
    ) -> impl ::core::future::Future<Output = ()> {
        <<Hall as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
            Hall,
        >>::call_entry_action(state, shared_storage, context)
    }
    fn exit_state<'ctx, 'data: 'ctx>(
        state: &mut <Self as ::sotu::private::Submachine>::State,
        shared_storage: &mut Self,
        context: &mut <Self as ::sotu::private::Submachine>::Context<'ctx, 'data>,
    ) -> impl ::core::future::Future<Output = ()> {
        <<Hall as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
            Hall,
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
        <<Hall as ::sotu::awaitable::IntoStateMachine>::Superstate<
            '_,
        > as ::sotu::awaitable::Superstate<
            Hall,
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
        <<Hall as ::sotu::awaitable::IntoStateMachine>::Superstate<
            '_,
        > as ::sotu::awaitable::Superstate<
            Hall,
        >>::call_exit_action(superstate, shared_storage, context)
    }
    fn before_dispatch<'ctx, 'data: 'ctx, 'sub>(
        shared_storage: &mut Self,
        node: ::sotu::StateOrSuperstate<
            '_,
            <Self as ::sotu::private::Submachine>::State,
            <Self as ::sotu::private::Submachine>::Superstate<'sub>,
        >,
        event: &<Self as ::sotu::private::Submachine>::Event,
        context: &mut <Self as ::sotu::private::Submachine>::Context<'ctx, 'data>,
    ) -> impl ::core::future::Future<Output = ()>
    where
        Self: 'sub,
    {
        <Hall as ::sotu::awaitable::IntoStateMachine>::before_dispatch(
            shared_storage,
            node,
            event,
            context,
        )
    }
    fn after_dispatch<'ctx, 'data: 'ctx, 'sub>(
        shared_storage: &mut Self,
        node: ::sotu::StateOrSuperstate<
            '_,
            <Self as ::sotu::private::Submachine>::State,
            <Self as ::sotu::private::Submachine>::Superstate<'sub>,
        >,
        event: &<Self as ::sotu::private::Submachine>::Event,
        context: &mut <Self as ::sotu::private::Submachine>::Context<'ctx, 'data>,
    ) -> impl ::core::future::Future<Output = ()>
    where
        Self: 'sub,
    {
        <Hall as ::sotu::awaitable::IntoStateMachine>::after_dispatch(
            shared_storage,
            node,
            event,
            context,
        )
    }
    fn before_transition<'ctx, 'data: 'ctx>(
        shared_storage: &mut Self,
        source: &<Self as ::sotu::private::Submachine>::State,
        target: &<Self as ::sotu::private::Submachine>::State,
        context: &mut <Self as ::sotu::private::Submachine>::Context<'ctx, 'data>,
    ) -> impl ::core::future::Future<Output = ()> {
        <Hall as ::sotu::awaitable::IntoStateMachine>::before_transition(
            shared_storage,
            source,
            target,
            context,
        )
    }
    fn after_transition<'ctx, 'data: 'ctx>(
        shared_storage: &mut Self,
        source: &<Self as ::sotu::private::Submachine>::State,
        target: &<Self as ::sotu::private::Submachine>::State,
        context: &mut <Self as ::sotu::private::Submachine>::Context<'ctx, 'data>,
    ) -> impl ::core::future::Future<Output = ()> {
        <Hall as ::sotu::awaitable::IntoStateMachine>::after_transition(
            shared_storage,
            source,
            target,
            context,
        )
    }
    fn internal_transition_nesting(
        source: &mut <Self as ::sotu::private::Submachine>::State,
        target: &mut <Self as ::sotu::private::Submachine>::State,
        exit_levels: usize,
    ) -> usize {
        <<Hall as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
            Hall,
        >>::__internal_transition_nesting(source, target, exit_levels)
    }
    fn before_internal_transition<'ctx, 'data: 'ctx>(
        source: &mut <Self as ::sotu::private::Submachine>::State,
        target: &mut <Self as ::sotu::private::Submachine>::State,
        exit_levels: usize,
        nesting: usize,
        shared_storage: &mut Self,
        context: &mut <Self as ::sotu::private::Submachine>::Context<'ctx, 'data>,
    ) -> impl ::core::future::Future<Output = ()> {
        <<Hall as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
            Hall,
        >>::__before_internal_transition(
            source,
            target,
            exit_levels,
            nesting,
            shared_storage,
            context,
        )
    }
    fn after_internal_transition<'ctx, 'data: 'ctx>(
        source: &mut <Self as ::sotu::private::Submachine>::State,
        target: &mut <Self as ::sotu::private::Submachine>::State,
        exit_levels: usize,
        nesting: usize,
        shared_storage: &mut Self,
        context: &mut <Self as ::sotu::private::Submachine>::Context<'ctx, 'data>,
    ) -> impl ::core::future::Future<Output = ()> {
        <<Hall as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
            Hall,
        >>::__after_internal_transition(
            source,
            target,
            exit_levels,
            nesting,
            shared_storage,
            context,
        )
    }
}
impl<'sub> Superstate<'sub>
where
    door::Door: ::sotu::private::AwaitableSubmachine,
{
    fn __depth(&self) -> usize {
        match self {
            Superstate::Loading(::sotu::SubmachineNode::Mount) => 1usize,
            Superstate::Loading(::sotu::SubmachineNode::Inner(superstate)) => {
                1usize
                    + <door::Door as ::sotu::private::AwaitableSubmachine>::superstate_depth(
                        superstate,
                    )
            }
        }
    }
}
impl Hall {
    #[doc(hidden)]
    #[allow(unreachable_code)]
    fn __project_into_loading<Visited>(
        event: Event,
        visit: impl ::core::ops::FnOnce(
            <door::Door as ::sotu::private::Submachine>::Event,
        ) -> (Visited, <door::Door as ::sotu::private::Submachine>::Event),
    ) -> ::core::result::Result<(Visited, Event), Event> {
        if !<<door::Door as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
            Event,
        >>::accepts(&event) {
            return ::core::result::Result::Err(event);
        }
        match <<door::Door as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
            Event,
        >>::project(event) {
            ::core::result::Result::Ok(child_event) => {
                let (visited, child_event) = visit(child_event);
                ::core::result::Result::Ok((
                    visited,
                    <<door::Door as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
                        Event,
                    >>::embed(child_event),
                ))
            }
            ::core::result::Result::Err(event) => ::core::result::Result::Err(event),
        }
    }
    #[doc(hidden)]
    #[allow(unreachable_code)]
    async fn __project_awaited_into_loading<Visited>(
        event: Event,
        visit: impl ::core::ops::AsyncFnOnce(
            <door::Door as ::sotu::private::Submachine>::Event,
        ) -> (Visited, <door::Door as ::sotu::private::Submachine>::Event),
    ) -> ::core::result::Result<(Visited, Event), Event> {
        if !<<door::Door as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
            Event,
        >>::accepts(&event) {
            return ::core::result::Result::Err(event);
        }
        match <<door::Door as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
            Event,
        >>::project(event) {
            ::core::result::Result::Ok(child_event) => {
                let (visited, child_event) = visit(child_event).await;
                ::core::result::Result::Ok((
                    visited,
                    <<door::Door as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
                        Event,
                    >>::embed(child_event),
                ))
            }
            ::core::result::Result::Err(event) => ::core::result::Result::Err(event),
        }
    }
}
fn main() {
    let _machine = Hall::default().state_machine();
}
