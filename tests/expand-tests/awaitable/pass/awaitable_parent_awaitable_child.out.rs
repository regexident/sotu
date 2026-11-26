use sotu::prelude::*;
mod door {
    use sotu::prelude::*;
    pub struct Open;
    pub enum Event {
        Open(Open),
    }
    impl ::core::convert::From<Open> for Event {
        fn from(value: Open) -> Self {
            Self::Open(value)
        }
    }
    impl ::sotu::event::Variant<Open> for Event {
        fn from_variant(value: Open) -> Self {
            Self::Open(value)
        }
        fn into_variant(self) -> ::core::result::Result<Open, Self> {
            match self {
                Self::Open(value) => ::core::result::Result::Ok(value),
            }
        }
        fn as_variant_ref(&self) -> ::core::option::Option<&Open> {
            match self {
                Self::Open(value) => ::core::option::Option::Some(value),
            }
        }
    }
    impl<SotuParentEvent> ::sotu::event::SubEvent<SotuParentEvent> for Event
    where
        SotuParentEvent: ::sotu::event::Variant<Open>,
    {
        fn accepts(parent: &SotuParentEvent) -> bool {
            <SotuParentEvent as ::sotu::event::Variant<Open>>::as_variant_ref(parent)
                .is_some()
        }
        fn project(
            parent: SotuParentEvent,
        ) -> ::core::result::Result<Self, SotuParentEvent> {
            let parent = match <SotuParentEvent as ::sotu::event::Variant<
                Open,
            >>::into_variant(parent) {
                ::core::result::Result::Ok(value) => {
                    return ::core::result::Result::Ok(Self::Open(value));
                }
                ::core::result::Result::Err(parent) => parent,
            };
            ::core::result::Result::Err(parent)
        }
        fn embed(self) -> SotuParentEvent {
            match self {
                Self::Open(value) => {
                    <SotuParentEvent as ::sotu::event::Variant<
                        Open,
                    >>::from_variant(value)
                }
            }
        }
    }
    pub struct Door {
        pub openings: u32,
    }
    impl Door {
        async fn closed(event: &Event) -> Outcome<State> {
            match event {
                Event::Open(_) => Transition(State::ajar()),
            }
        }
        fn ajar(event: &Event) -> Outcome<State> {
            match event {
                Event::Open(_) => Super,
            }
        }
        async fn frame() -> Outcome<State> {
            Handled
        }
        async fn count(&mut self) {
            self.openings += 1;
        }
    }
    pub enum State {
        Closed {},
        Ajar {},
    }
    impl State {
        pub fn closed() -> Self {
            Self::Closed {}
        }
        pub fn ajar() -> Self {
            Self::Ajar {}
        }
    }
    pub enum Superstate {
        Frame {},
    }
    impl ::sotu::awaitable::IntoStateMachine for Door {
        type Event = Event;
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = Superstate where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::closed()
        }
    }
    impl ::sotu::awaitable::State<Door> for State {
        fn call_handler<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Door,
            event: &Event,
            _context: &mut (),
        ) -> impl ::core::future::Future<
            Output = ::sotu::Outcome<Self, ::core::convert::Infallible>,
        > {
            async move {
                match self {
                    State::Closed {} => Door::closed(event).await,
                    State::Ajar {} => Door::ajar(event),
                }
            }
        }
        fn call_entry_action<'ctx, 'data: 'ctx>(
            &mut self,
            shared_storage: &mut Door,
            _context: &mut (),
        ) -> impl ::core::future::Future<Output = ()> {
            async move {
                match self {
                    State::Closed {} => Door::count(shared_storage).await,
                    State::Ajar {} => {}
                }
            }
        }
        fn superstate(&mut self) -> ::core::option::Option<Superstate> {
            match self {
                State::Closed {} => ::core::option::Option::Some(Superstate::Frame {}),
                State::Ajar {} => ::core::option::Option::Some(Superstate::Frame {}),
            }
        }
    }
    impl ::sotu::awaitable::Superstate<Door> for Superstate {
        fn call_handler<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Door,
            _event: &Event,
            _context: &mut (),
        ) -> impl ::core::future::Future<
            Output = ::sotu::Outcome<State, ::core::convert::Infallible>,
        > {
            async move {
                match self {
                    Superstate::Frame {} => Door::frame().await,
                }
            }
        }
        fn superstate(self) -> ::core::option::Option<Self> {
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
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::closed()
        }
    }
    impl ::sotu::private::ExitlessSubmachine for Door {}
    impl ::sotu::private::AwaitableSubmachine for Door {
        fn state_superstate<'sub>(
            state: &'sub mut <Self as ::sotu::private::Submachine>::State,
        ) -> ::core::option::Option<
            <Self as ::sotu::private::Submachine>::Superstate<'sub>,
        >
        where
            Self: 'sub,
        {
            <<Door as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
                Door,
            >>::superstate(state)
        }
        fn state_same(
            first: &<Self as ::sotu::private::Submachine>::State,
            second: &<Self as ::sotu::private::Submachine>::State,
        ) -> bool {
            <<Door as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
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
            <<Door as ::sotu::awaitable::IntoStateMachine>::Superstate<
                '_,
            > as ::sotu::awaitable::Superstate<Door>>::same_state(first, second)
        }
        fn state_depth(
            state: &mut <Self as ::sotu::private::Submachine>::State,
        ) -> usize {
            <<Door as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::StateExt<
                Door,
            >>::depth(state)
        }
        fn state_is_addressed(
            state: &<Self as ::sotu::private::Submachine>::State,
            event: <Self as ::sotu::private::Submachine>::Event,
        ) -> (bool, <Self as ::sotu::private::Submachine>::Event) {
            <<Door as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
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
            <<Door as ::sotu::awaitable::IntoStateMachine>::Superstate<
                '_,
            > as ::sotu::awaitable::Superstate<Door>>::__is_addressed(superstate, event)
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
            <<Door as ::sotu::awaitable::IntoStateMachine>::Superstate<
                '_,
            > as ::sotu::awaitable::Superstate<Door>>::superstate(superstate)
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
            <<Door as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
                Door,
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
            <<Door as ::sotu::awaitable::IntoStateMachine>::Superstate<
                '_,
            > as ::sotu::awaitable::Superstate<
                Door,
            >>::__dispatch(superstate, shared_storage, event, context)
        }
        fn enter_state<'ctx, 'data: 'ctx>(
            state: &mut <Self as ::sotu::private::Submachine>::State,
            shared_storage: &mut Self,
            context: &mut <Self as ::sotu::private::Submachine>::Context<'ctx, 'data>,
        ) -> impl ::core::future::Future<Output = ()> {
            <<Door as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
                Door,
            >>::call_entry_action(state, shared_storage, context)
        }
        fn exit_state<'ctx, 'data: 'ctx>(
            state: &mut <Self as ::sotu::private::Submachine>::State,
            shared_storage: &mut Self,
            context: &mut <Self as ::sotu::private::Submachine>::Context<'ctx, 'data>,
        ) -> impl ::core::future::Future<Output = ()> {
            <<Door as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
                Door,
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
            <<Door as ::sotu::awaitable::IntoStateMachine>::Superstate<
                '_,
            > as ::sotu::awaitable::Superstate<
                Door,
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
            <<Door as ::sotu::awaitable::IntoStateMachine>::Superstate<
                '_,
            > as ::sotu::awaitable::Superstate<
                Door,
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
            <Door as ::sotu::awaitable::IntoStateMachine>::before_dispatch(
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
            <Door as ::sotu::awaitable::IntoStateMachine>::after_dispatch(
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
            <Door as ::sotu::awaitable::IntoStateMachine>::before_transition(
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
            <Door as ::sotu::awaitable::IntoStateMachine>::after_transition(
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
            <<Door as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
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
            <<Door as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
                Door,
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
            <<Door as ::sotu::awaitable::IntoStateMachine>::State as ::sotu::awaitable::State<
                Door,
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
    impl Superstate {
        fn __depth(&self) -> usize {
            match self {
                Superstate::Frame { .. } => 1usize,
            }
        }
    }
}
pub enum Event {
    Open(door::Open),
}
impl ::core::convert::From<door::Open> for Event {
    fn from(value: door::Open) -> Self {
        Self::Open(value)
    }
}
impl ::sotu::event::Variant<door::Open> for Event {
    fn from_variant(value: door::Open) -> Self {
        Self::Open(value)
    }
    fn into_variant(self) -> ::core::result::Result<door::Open, Self> {
        match self {
            Self::Open(value) => ::core::result::Result::Ok(value),
        }
    }
    fn as_variant_ref(&self) -> ::core::option::Option<&door::Open> {
        match self {
            Self::Open(value) => ::core::option::Option::Some(value),
        }
    }
}
impl<SotuParentEvent> ::sotu::event::SubEvent<SotuParentEvent> for Event
where
    SotuParentEvent: ::sotu::event::Variant<door::Open>,
{
    fn accepts(parent: &SotuParentEvent) -> bool {
        <SotuParentEvent as ::sotu::event::Variant<door::Open>>::as_variant_ref(parent)
            .is_some()
    }
    fn project(
        parent: SotuParentEvent,
    ) -> ::core::result::Result<Self, SotuParentEvent> {
        let parent = match <SotuParentEvent as ::sotu::event::Variant<
            door::Open,
        >>::into_variant(parent) {
            ::core::result::Result::Ok(value) => {
                return ::core::result::Result::Ok(Self::Open(value));
            }
            ::core::result::Result::Err(parent) => parent,
        };
        ::core::result::Result::Err(parent)
    }
    fn embed(self) -> SotuParentEvent {
        match self {
            Self::Open(value) => {
                <SotuParentEvent as ::sotu::event::Variant<
                    door::Open,
                >>::from_variant(value)
            }
        }
    }
}
pub struct Hall {
    pub door: Option<door::Door>,
    pub trace: Vec<&'static str>,
}
impl Hall {
    fn make_door(&mut self, _context: &mut ()) -> door::Door {
        self.trace.push("make door");
        door::Door { openings: 0 }
    }
}
impl Hall {
    async fn before(
        &mut self,
        _node: StateOrSuperstate<'_, State, Superstate<'_>>,
        _event: &Event,
    ) {
        self.trace.push("before");
    }
    fn after(&mut self, _source: &State, _target: &State) {
        self.trace.push("after");
    }
    fn loading(event: &Event) -> Outcome<State> {
        match event {
            Event::Open(_) => Handled,
        }
    }
    fn region() -> Outcome<State> {
        Handled
    }
}
pub enum State {
    Loading(<door::Door as ::sotu::private::Submachine>::State),
}
impl State {
    pub fn loading(state: <door::Door as ::sotu::private::Submachine>::State) -> Self {
        Self::Loading(state)
    }
}
pub enum Superstate<'sub> {
    Region {},
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
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        Superstate::loading()
    }
    fn before_dispatch<'ctx, 'data: 'ctx>(
        &mut self,
        node: ::sotu::StateOrSuperstate<
            '_,
            State,
            <Self as ::sotu::awaitable::IntoStateMachine>::Superstate<'_>,
        >,
        event: &Event,
        context: &mut (),
    ) -> impl ::core::future::Future<Output = ()> {
        ::sotu::awaitable::DispatchHook::call_dispatch_hook(
            &(Self::before),
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
        context: &mut (),
    ) -> impl ::core::future::Future<Output = ()> {
        ::sotu::awaitable::TransitionHook::call_transition_hook(
            &(Self::after),
            self,
            source,
            target,
            context,
        )
    }
}
impl ::sotu::awaitable::State<Hall> for State
where
    door::Door: ::sotu::private::AwaitableSubmachine,
    door::Door: ::sotu::private::ExitlessSubmachine,
{
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Hall,
        _event: &Event,
        _context: &mut (),
    ) -> impl ::core::future::Future<
        Output = ::sotu::Outcome<Self, ::core::convert::Infallible>,
    > {
        async move {
            match self {
                State::Loading(_) => ::sotu::Outcome::Super,
            }
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Hall,
        context: &mut (),
    ) -> impl ::core::future::Future<Output = ()> {
        async move {
            match self {
                State::Loading(state) => {
                    if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                        door::Door,
                    >::get_mut(&mut shared_storage.door) {
                        <door::Door as ::sotu::private::AwaitableSubmachine>::enter_state(
                                state,
                                submachine,
                                context,
                            )
                            .await;
                    }
                }
            }
        }
    }
    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Hall,
        context: &mut (),
    ) -> impl ::core::future::Future<Output = ()> {
        async move {
            match self {
                State::Loading(state) => {
                    if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                        door::Door,
                    >::get_mut(&mut shared_storage.door) {
                        <door::Door as ::sotu::private::AwaitableSubmachine>::exit_state(
                                state,
                                submachine,
                                context,
                            )
                            .await;
                    }
                }
            }
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate<'_>> {
        match self {
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
            (State::Loading(first), State::Loading(second)) => {
                <door::Door as ::sotu::private::AwaitableSubmachine>::state_same(
                    first,
                    second,
                )
            }
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
        }
    }
    #[allow(unreachable_code)]
    fn __dispatch<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Hall,
        event: Event,
        context: &mut (),
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
                                <door::Door as ::sotu::private::AwaitableSubmachine>::before_dispatch(
                                        submachine,
                                        ::sotu::StateOrSuperstate::State(&*node),
                                        &event,
                                        context,
                                    )
                                    .await;
                                let ::sotu::private::Dispatched { outcome, event } = <door::Door as ::sotu::private::AwaitableSubmachine>::dispatch_state(
                                        node,
                                        submachine,
                                        event,
                                        context,
                                    )
                                    .await;
                                <door::Door as ::sotu::private::AwaitableSubmachine>::after_dispatch(
                                        submachine,
                                        ::sotu::StateOrSuperstate::State(&*node),
                                        &event,
                                        context,
                                    )
                                    .await;
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
                                    ::sotu::private::NodeOutcome::Exit(exit) => match exit {}
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
        }
    }
    fn __before_internal_transition<'ctx, 'data: 'ctx>(
        &mut self,
        target: &mut Self,
        exit_levels: usize,
        nesting: usize,
        shared_storage: &mut Hall,
        context: &mut (),
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
                            <door::Door as ::sotu::private::AwaitableSubmachine>::before_transition(
                                    submachine,
                                    source,
                                    target,
                                    context,
                                )
                                .await;
                        }
                        ::core::option::Option::Some(deeper) => {
                            <door::Door as ::sotu::private::AwaitableSubmachine>::before_internal_transition(
                                    source,
                                    target,
                                    exit_levels,
                                    deeper,
                                    submachine,
                                    context,
                                )
                                .await;
                        }
                        ::core::option::Option::None => {}
                    }
                }
            }
        }
    }
    fn __after_internal_transition<'ctx, 'data: 'ctx>(
        &mut self,
        target: &mut Self,
        exit_levels: usize,
        nesting: usize,
        shared_storage: &mut Hall,
        context: &mut (),
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
                            <door::Door as ::sotu::private::AwaitableSubmachine>::after_transition(
                                    submachine,
                                    source,
                                    target,
                                    context,
                                )
                                .await;
                        }
                        ::core::option::Option::Some(deeper) => {
                            <door::Door as ::sotu::private::AwaitableSubmachine>::after_internal_transition(
                                    source,
                                    target,
                                    exit_levels,
                                    deeper,
                                    submachine,
                                    context,
                                )
                                .await;
                        }
                        ::core::option::Option::None => {}
                    }
                }
            }
        }
    }
}
impl<'sub> ::sotu::awaitable::Superstate<Hall> for Superstate<'sub>
where
    door::Door: ::sotu::private::AwaitableSubmachine,
    door::Door: ::sotu::private::ExitlessSubmachine,
{
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Hall,
        event: &Event,
        _context: &mut (),
    ) -> impl ::core::future::Future<
        Output = ::sotu::Outcome<State, ::core::convert::Infallible>,
    > {
        async move {
            match self {
                Superstate::Region {} => Hall::region(),
                Superstate::Loading(::sotu::SubmachineNode::Inner(_)) => {
                    ::sotu::Outcome::Super
                }
                Superstate::Loading(::sotu::SubmachineNode::Mount) => {
                    Hall::loading(event)
                }
            }
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Hall,
        context: &mut (),
    ) -> impl ::core::future::Future<Output = ()> {
        async move {
            match self {
                Superstate::Region { .. } => {}
                Superstate::Loading(::sotu::SubmachineNode::Mount) => {
                    if ::sotu::private::FactoryInstance::<
                        door::Door,
                    >::is_vacant(&shared_storage.door) {
                        let instance = (Hall::make_door)(shared_storage, context);
                        ::sotu::private::FactoryInstance::<
                            door::Door,
                        >::install(&mut shared_storage.door, instance);
                    }
                }
                Superstate::Loading(::sotu::SubmachineNode::Inner(state)) => {
                    if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                        door::Door,
                    >::get_mut(&mut shared_storage.door) {
                        <door::Door as ::sotu::private::AwaitableSubmachine>::enter_superstate(
                                state,
                                submachine,
                                context,
                            )
                            .await;
                    }
                }
            }
        }
    }
    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Hall,
        context: &mut (),
    ) -> impl ::core::future::Future<Output = ()> {
        async move {
            match self {
                Superstate::Region { .. } => {}
                Superstate::Loading(::sotu::SubmachineNode::Mount) => {
                    ::sotu::private::MountField::<
                        door::Door,
                    >::release(&mut shared_storage.door)
                }
                Superstate::Loading(::sotu::SubmachineNode::Inner(state)) => {
                    if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                        door::Door,
                    >::get_mut(&mut shared_storage.door) {
                        <door::Door as ::sotu::private::AwaitableSubmachine>::exit_superstate(
                                state,
                                submachine,
                                context,
                            )
                            .await;
                    }
                }
            }
        }
    }
    fn superstate(self) -> ::core::option::Option<Self> {
        match self {
            Superstate::Region {} => ::core::option::Option::None,
            Superstate::Loading(::sotu::SubmachineNode::Mount) => {
                ::core::option::Option::Some(Superstate::Region {})
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
            (Superstate::Region { .. }, Superstate::Region { .. }) => true,
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
        context: &mut (),
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
                                <door::Door as ::sotu::private::AwaitableSubmachine>::before_dispatch(
                                        submachine,
                                        ::sotu::StateOrSuperstate::Superstate(&*node),
                                        &event,
                                        context,
                                    )
                                    .await;
                                let ::sotu::private::Dispatched { outcome, event } = <door::Door as ::sotu::private::AwaitableSubmachine>::dispatch_superstate(
                                        node,
                                        submachine,
                                        event,
                                        context,
                                    )
                                    .await;
                                <door::Door as ::sotu::private::AwaitableSubmachine>::after_dispatch(
                                        submachine,
                                        ::sotu::StateOrSuperstate::Superstate(&*node),
                                        &event,
                                        context,
                                    )
                                    .await;
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
                                    ::sotu::private::NodeOutcome::Exit(exit) => match exit {}
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
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        Superstate::loading()
    }
}
impl ::sotu::private::ExitlessSubmachine for Hall {}
impl ::sotu::private::AwaitableSubmachine for Hall
where
    door::Door: ::sotu::private::AwaitableSubmachine,
    door::Door: ::sotu::private::ExitlessSubmachine,
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
            Superstate::Region { .. } => 1usize,
            Superstate::Loading(::sotu::SubmachineNode::Mount) => 2usize,
            Superstate::Loading(::sotu::SubmachineNode::Inner(superstate)) => {
                2usize
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
    let _machine = Hall {
        door: None,
        trace: Vec::new(),
    }
        .state_machine();
}
