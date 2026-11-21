use sotu::prelude::*;
mod door {
    use sotu::prelude::*;
    pub struct Door {
        pub openings: u32,
    }
    #[automatically_derived]
    impl ::core::default::Default for Door {
        #[inline]
        fn default() -> Door {
            Door {
                openings: ::core::default::Default::default(),
            }
        }
    }
    pub struct Close;
    pub enum Event {
        Close(Close),
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
            }
        }
        fn as_variant_ref(&self) -> ::core::option::Option<&Close> {
            match self {
                Self::Close(value) => ::core::option::Option::Some(value),
            }
        }
    }
    impl<SotuParentEvent> ::sotu::event::SubEvent<SotuParentEvent> for Event
    where
        SotuParentEvent: ::sotu::event::Variant<Close>,
    {
        fn accepts(parent: &SotuParentEvent) -> bool {
            <SotuParentEvent as ::sotu::event::Variant<Close>>::as_variant_ref(parent)
                .is_some()
        }
        fn project(
            parent: SotuParentEvent,
        ) -> ::core::result::Result<Self, SotuParentEvent> {
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
                Self::Close(value) => {
                    <SotuParentEvent as ::sotu::event::Variant<
                        Close,
                    >>::from_variant(value)
                }
            }
        }
    }
    pub enum DoorExit {
        Closed,
    }
    impl Door {
        fn opened(event: &Event) -> Outcome<State, DoorExit> {
            match event {
                Event::Close(_) => Super,
            }
        }
        /// The door exits from its superstate as well as from its leaves.
        fn frame(event: &Event) -> Outcome<State, DoorExit> {
            match event {
                Event::Close(_) => Exit(DoorExit::Closed),
            }
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
        /// The door exits from its superstate as well as from its leaves.
        Frame {},
    }
    impl ::sotu::blocking::IntoStateMachine for Door {
        type Event = Event;
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = Superstate where Self: 'sub;
        type Exit = DoorExit;
        fn initial() -> State {
            State::opened()
        }
    }
    impl ::sotu::blocking::State<Door> for State {
        fn call_handler<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Door,
            event: &Event,
            _context: &mut (),
        ) -> ::sotu::Outcome<Self, DoorExit> {
            match self {
                State::Opened {} => Door::opened(event),
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
            event: &Event,
            _context: &mut (),
        ) -> ::sotu::Outcome<State, DoorExit> {
            match self {
                Superstate::Frame {} => Door::frame(event),
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
        type Exit = DoorExit;
        fn initial() -> State {
            State::opened()
        }
    }
    impl ::sotu::private::BlockingSubmachine for Door {}
}
pub struct Start;
pub enum Event {
    Start(Start),
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
    SotuParentEvent: ::sotu::event::Variant<door::Close>,
{
    fn accepts(parent: &SotuParentEvent) -> bool {
        <SotuParentEvent as ::sotu::event::Variant<Start>>::as_variant_ref(parent)
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
            Self::Close(value) => {
                <SotuParentEvent as ::sotu::event::Variant<
                    door::Close,
                >>::from_variant(value)
            }
        }
    }
}
pub struct House {
    door: Option<door::Door>,
}
#[automatically_derived]
impl ::core::default::Default for House {
    #[inline]
    fn default() -> House {
        House {
            door: ::core::default::Default::default(),
        }
    }
}
impl House {
    fn idle(event: &Event) -> Outcome<State> {
        match event {
            Event::Start(_) => Transition(Superstate::loading()),
            Event::Close(_) => Handled,
        }
    }
    /// The door's exits go to `door_exited`.
    fn loading(event: &Event) -> Outcome<State> {
        match event {
            Event::Start(_) => Handled,
            Event::Close(_) => Super,
        }
    }
    fn door_exited(submachine: &mut door::Door, exit: door::DoorExit) -> Outcome<State> {
        match exit {
            door::DoorExit::Closed if submachine.openings > 0 => {
                Transition(State::idle())
            }
            door::DoorExit::Closed => Super,
        }
    }
}
pub enum State {
    Idle {},
    /// The door's exits go to `door_exited`.
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
    /// The door's exits go to `door_exited`.
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
impl ::sotu::blocking::IntoStateMachine for House {
    type Event = Event;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::idle()
    }
}
impl ::sotu::blocking::State<House> for State
where
    door::Door: ::sotu::private::BlockingSubmachine,
{
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut House,
        event: &Event,
        _context: &mut (),
    ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
        match self {
            State::Idle {} => House::idle(event),
            State::Loading(_) => ::sotu::Outcome::Super,
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut House,
        context: &mut (),
    ) {
        match self {
            State::Idle { .. } => {}
            State::Loading(state) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    door::Door,
                >::get_mut(&mut shared_storage.door) {
                    <<door::Door as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                        door::Door,
                    >>::call_entry_action(state, submachine, context);
                }
            }
        }
    }
    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut House,
        context: &mut (),
    ) {
        match self {
            State::Idle { .. } => {}
            State::Loading(state) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    door::Door,
                >::get_mut(&mut shared_storage.door) {
                    <<door::Door as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                        door::Door,
                    >>::call_exit_action(state, submachine, context);
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
                        match <<door::Door as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            door::Door,
                        >>::superstate(state) {
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
                <<door::Door as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                    door::Door,
                >>::same_state(first, second)
            }
            _ => false,
        }
    }
    fn __is_addressed(&self, event: Event) -> (bool, Event) {
        match self {
            State::Loading(node) => {
                match House::__project_into_loading(
                    event,
                    |event| {
                        <<door::Door as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            door::Door,
                        >>::__is_addressed(node, event)
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
        shared_storage: &mut House,
        event: Event,
        context: &mut (),
    ) -> ::sotu::private::Dispatched<Self, ::core::convert::Infallible, Event> {
        match self {
            State::Loading(node) => {
                let mount_offset = <<door::Door as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::StateExt<
                    door::Door,
                >>::depth(node);
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
                let visited = House::__project_into_loading(
                    event,
                    |event| {
                        <door::Door as ::sotu::blocking::IntoStateMachine>::before_dispatch(
                            submachine,
                            ::sotu::StateOrSuperstate::State(&*node),
                            &event,
                            context,
                        );
                        let ::sotu::private::Dispatched { outcome, event } = <<door::Door as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            door::Door,
                        >>::__dispatch(node, submachine, event, context);
                        <door::Door as ::sotu::blocking::IntoStateMachine>::after_dispatch(
                            submachine,
                            ::sotu::StateOrSuperstate::State(&*node),
                            &event,
                            context,
                        );
                        (outcome, event)
                    },
                );
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
                                    match ::sotu::private::MountField::<
                                        door::Door,
                                    >::get_mut(&mut shared_storage.door) {
                                        ::core::option::Option::Some(submachine) => {
                                            match House::door_exited(submachine, exit) {
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
                                        ::core::option::Option::None => {
                                            ::sotu::private::NodeOutcome::DeferToMount {
                                                mount_offset,
                                            }
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
                let outcome = ::sotu::blocking::State::<
                    House,
                >::call_handler(self, shared_storage, &event, context);
                ::sotu::private::Dispatched {
                    outcome: ::sotu::private::NodeOutcome::from(outcome),
                    event: event,
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
                    <= <<door::Door as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::StateExt<
                        door::Door,
                    >>::depth(source)
                {
                    1
                        + <<door::Door as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            door::Door,
                        >>::__internal_transition_nesting(source, target, exit_levels)
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
        shared_storage: &mut House,
        context: &mut (),
    ) {
        match (self, target) {
            (State::Loading(source), State::Loading(target)) => {
                if exit_levels
                    > <<door::Door as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::StateExt<
                        door::Door,
                    >>::depth(source)
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
                        <door::Door as ::sotu::blocking::IntoStateMachine>::before_transition(
                            submachine,
                            source,
                            target,
                            context,
                        );
                    }
                    ::core::option::Option::Some(deeper) => {
                        <<door::Door as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            door::Door,
                        >>::__before_internal_transition(
                            source,
                            target,
                            exit_levels,
                            deeper,
                            submachine,
                            context,
                        );
                    }
                    ::core::option::Option::None => {}
                }
            }
            _ => {}
        }
    }
    fn __after_internal_transition<'ctx, 'data: 'ctx>(
        &mut self,
        target: &mut Self,
        exit_levels: usize,
        nesting: usize,
        shared_storage: &mut House,
        context: &mut (),
    ) {
        match (self, target) {
            (State::Loading(source), State::Loading(target)) => {
                if exit_levels
                    > <<door::Door as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::StateExt<
                        door::Door,
                    >>::depth(source)
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
                        <door::Door as ::sotu::blocking::IntoStateMachine>::after_transition(
                            submachine,
                            source,
                            target,
                            context,
                        );
                    }
                    ::core::option::Option::Some(deeper) => {
                        <<door::Door as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            door::Door,
                        >>::__after_internal_transition(
                            source,
                            target,
                            exit_levels,
                            deeper,
                            submachine,
                            context,
                        );
                    }
                    ::core::option::Option::None => {}
                }
            }
            _ => {}
        }
    }
}
impl<'sub> ::sotu::blocking::Superstate<House> for Superstate<'sub>
where
    door::Door: ::sotu::private::BlockingSubmachine,
{
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut House,
        event: &Event,
        _context: &mut (),
    ) -> ::sotu::Outcome<State, ::core::convert::Infallible> {
        match self {
            Superstate::Loading(::sotu::SubmachineNode::Inner(_)) => {
                ::sotu::Outcome::Super
            }
            Superstate::Loading(::sotu::SubmachineNode::Mount) => House::loading(event),
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut House,
        context: &mut (),
    ) {
        match self {
            Superstate::Loading(::sotu::SubmachineNode::Mount) => {
                ::sotu::private::DefaultInstance::<
                    door::Door,
                >::fill_default(&mut shared_storage.door)
            }
            Superstate::Loading(::sotu::SubmachineNode::Inner(state)) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    door::Door,
                >::get_mut(&mut shared_storage.door) {
                    <<door::Door as ::sotu::blocking::IntoStateMachine>::Superstate<
                        '_,
                    > as ::sotu::blocking::Superstate<
                        door::Door,
                    >>::call_entry_action(state, submachine, context);
                }
            }
        }
    }
    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut House,
        context: &mut (),
    ) {
        match self {
            Superstate::Loading(::sotu::SubmachineNode::Mount) => {
                ::sotu::private::MountField::<
                    door::Door,
                >::release(&mut shared_storage.door)
            }
            Superstate::Loading(::sotu::SubmachineNode::Inner(state)) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    door::Door,
                >::get_mut(&mut shared_storage.door) {
                    <<door::Door as ::sotu::blocking::IntoStateMachine>::Superstate<
                        '_,
                    > as ::sotu::blocking::Superstate<
                        door::Door,
                    >>::call_exit_action(state, submachine, context);
                }
            }
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate<'_>> {
        match self {
            Superstate::Loading(::sotu::SubmachineNode::Mount) => {
                ::core::option::Option::None
            }
            Superstate::Loading(::sotu::SubmachineNode::Inner(superstate)) => {
                ::core::option::Option::Some(
                    Superstate::Loading(
                        match <<door::Door as ::sotu::blocking::IntoStateMachine>::Superstate<
                            '_,
                        > as ::sotu::blocking::Superstate<
                            door::Door,
                        >>::superstate(superstate) {
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
                <<door::Door as ::sotu::blocking::IntoStateMachine>::Superstate<
                    '_,
                > as ::sotu::blocking::Superstate<door::Door>>::same_state(first, second)
            }
            _ => false,
        }
    }
    fn __is_addressed(&self, event: Event) -> (bool, Event) {
        match self {
            Superstate::Loading(::sotu::SubmachineNode::Inner(node)) => {
                match House::__project_into_loading(
                    event,
                    |event| {
                        <<door::Door as ::sotu::blocking::IntoStateMachine>::Superstate<
                            '_,
                        > as ::sotu::blocking::Superstate<
                            door::Door,
                        >>::__is_addressed(node, event)
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
        shared_storage: &mut House,
        event: Event,
        context: &mut (),
    ) -> ::sotu::private::Dispatched<State, ::core::convert::Infallible, Event> {
        match self {
            Superstate::Loading(::sotu::SubmachineNode::Inner(node)) => {
                let mount_offset = <<door::Door as ::sotu::blocking::IntoStateMachine>::Superstate<
                    '_,
                > as ::sotu::blocking::SuperstateExt<door::Door>>::depth(node);
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
                let visited = House::__project_into_loading(
                    event,
                    |event| {
                        <door::Door as ::sotu::blocking::IntoStateMachine>::before_dispatch(
                            submachine,
                            ::sotu::StateOrSuperstate::Superstate(&*node),
                            &event,
                            context,
                        );
                        let ::sotu::private::Dispatched { outcome, event } = <<door::Door as ::sotu::blocking::IntoStateMachine>::Superstate<
                            '_,
                        > as ::sotu::blocking::Superstate<
                            door::Door,
                        >>::__dispatch(node, submachine, event, context);
                        <door::Door as ::sotu::blocking::IntoStateMachine>::after_dispatch(
                            submachine,
                            ::sotu::StateOrSuperstate::Superstate(&*node),
                            &event,
                            context,
                        );
                        (outcome, event)
                    },
                );
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
                                    match ::sotu::private::MountField::<
                                        door::Door,
                                    >::get_mut(&mut shared_storage.door) {
                                        ::core::option::Option::Some(submachine) => {
                                            match House::door_exited(submachine, exit) {
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
                                        ::core::option::Option::None => {
                                            ::sotu::private::NodeOutcome::DeferToMount {
                                                mount_offset,
                                            }
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
                let outcome = ::sotu::blocking::Superstate::<
                    House,
                >::call_handler(self, shared_storage, &event, context);
                ::sotu::private::Dispatched {
                    outcome: ::sotu::private::NodeOutcome::from(outcome),
                    event: event,
                }
            }
        }
    }
}
impl ::sotu::private::Submachine for House {
    type Event = Event;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::idle()
    }
}
impl ::sotu::private::BlockingSubmachine for House {}
impl ::sotu::private::ExitlessSubmachine for House {}
impl House {
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
}
fn main() {
    let mut machine = House::default().state_machine();
    machine.handle(Start, &mut ());
    machine.handle(door::Close, &mut ());
}
