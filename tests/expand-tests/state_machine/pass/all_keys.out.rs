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
                Event::Close(_) => Exit(DoorExit::Closed),
            }
        }
    }
    pub enum State {
        Opened {},
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for State {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(f, "Opened")
        }
    }
    impl State {
        pub fn opened() -> Self {
            Self::Opened {}
        }
    }
    impl ::sotu::blocking::IntoStateMachine for Door {
        type Event = Event;
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = () where Self: 'sub;
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
    }
    impl ::sotu::private::Submachine for Door {
        type Event = Event;
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = () where Self: 'sub;
        type Exit = DoorExit;
        fn initial() -> State {
            State::opened()
        }
    }
    impl ::sotu::private::BlockingSubmachine for Door {}
}
pub struct Sink<'ctx, 'data> {
    pub output: &'ctx mut Vec<&'data str>,
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
pub enum Done {
    Washed,
}
pub(crate) struct Washer {
    door: Option<door::Door>,
}
#[automatically_derived]
impl ::core::default::Default for Washer {
    #[inline]
    fn default() -> Washer {
        Washer {
            door: ::core::default::Default::default(),
        }
    }
}
impl Washer {
    fn before_dispatch(
        &mut self,
        _node: StateOrSuperstate<'_, Mode, Group<'_>>,
        _input: &Event,
        sink: &mut Sink<'_, '_>,
    ) {
        sink.output.push("before dispatch");
    }
    fn after_dispatch(
        &mut self,
        _node: StateOrSuperstate<'_, Mode, Group<'_>>,
        _input: &Event,
    ) {}
    fn before_transition(
        &mut self,
        _source: &Mode,
        _target: &Mode,
        sink: &mut Sink<'_, '_>,
    ) {
        sink.output.push("before transition");
    }
    fn after_transition(&mut self, _source: &Mode, _target: &Mode) {}
    fn idle(input: &Event) -> Outcome<Mode, Done> {
        match input {
            Event::Start(_) => Transition(Group::loading()),
            Event::Close(_) => Handled,
        }
    }
    fn loading(child: &mut door::Door, input: &Event) -> Outcome<Mode, Done> {
        match input {
            Event::Start(_) if child.openings > 0 => Handled,
            _ => Super,
        }
    }
    fn door_exited(
        child: &mut door::Door,
        reason: door::DoorExit,
        sink: &mut Sink<'_, '_>,
    ) -> Outcome<Mode, Done> {
        sink.output.push("door exited");
        match reason {
            door::DoorExit::Closed if child.openings > 0 => Transition(Mode::idle()),
            door::DoorExit::Closed => Exit(Done::Washed),
        }
    }
}
pub(crate) enum Mode {
    Idle {},
    Loading(<door::Door as ::sotu::private::Submachine>::State),
}
#[automatically_derived]
impl ::core::fmt::Debug for Mode {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            Mode::Idle {} => ::core::fmt::Formatter::write_str(f, "Idle"),
            Mode::Loading(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "Loading",
                    &__self_0,
                )
            }
        }
    }
}
impl Mode {
    pub(crate) fn idle() -> Self {
        Self::Idle {}
    }
    pub(crate) fn loading(
        state: <door::Door as ::sotu::private::Submachine>::State,
    ) -> Self {
        Self::Loading(state)
    }
}
pub(crate) enum Group<'sub> {
    Loading(
        ::sotu::SubmachineNode<
            <door::Door as ::sotu::private::Submachine>::Superstate<'sub>,
        >,
    ),
}
#[automatically_derived]
impl<'sub> ::core::fmt::Debug for Group<'sub> {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            Group::Loading(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "Loading",
                    &__self_0,
                )
            }
        }
    }
}
impl Group<'_> {
    pub(crate) fn loading() -> Mode {
        Mode::Loading(<door::Door as ::sotu::private::Submachine>::initial())
    }
}
impl ::sotu::blocking::IntoStateMachine for Washer {
    type Event = Event;
    type Context<'ctx, 'data: 'ctx> = Sink<'ctx, 'data>;
    type State = Mode;
    type Superstate<'sub> = Group<'sub> where Self: 'sub;
    type Exit = Done;
    fn initial() -> Mode {
        Mode::idle()
    }
    fn before_dispatch<'ctx, 'data: 'ctx>(
        &mut self,
        node: ::sotu::StateOrSuperstate<'_, Mode, Group<'_>>,
        event: &Event,
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
    fn after_dispatch<'ctx, 'data: 'ctx>(
        &mut self,
        node: ::sotu::StateOrSuperstate<'_, Mode, Group<'_>>,
        event: &Event,
        context: &mut Sink<'ctx, 'data>,
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
        source: &Mode,
        target: &Mode,
        context: &mut Sink<'ctx, 'data>,
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
        source: &Mode,
        target: &Mode,
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
impl ::sotu::blocking::State<Washer> for Mode
where
    door::Door: ::sotu::private::BlockingSubmachine,
{
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Washer,
        event: &Event,
        _context: &mut Sink<'ctx, 'data>,
    ) -> ::sotu::Outcome<Self, Done> {
        match self {
            Mode::Idle {} => Washer::idle(event),
            Mode::Loading(_) => ::sotu::Outcome::Super,
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Washer,
        context: &mut Sink<'ctx, 'data>,
    ) {
        match self {
            Mode::Idle { .. } => {}
            Mode::Loading(state) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    door::Door,
                >::get_mut(&mut shared_storage.door) {
                    {
                        let mut child_context = (sotu::event::unit_context)(context);
                        <<door::Door as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            door::Door,
                        >>::call_entry_action(state, submachine, &mut child_context)
                    };
                }
            }
        }
    }
    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Washer,
        context: &mut Sink<'ctx, 'data>,
    ) {
        match self {
            Mode::Idle { .. } => {}
            Mode::Loading(state) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    door::Door,
                >::get_mut(&mut shared_storage.door) {
                    {
                        let mut child_context = (sotu::event::unit_context)(context);
                        <<door::Door as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            door::Door,
                        >>::call_exit_action(state, submachine, &mut child_context)
                    };
                }
            }
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Group<'_>> {
        match self {
            Mode::Idle {} => ::core::option::Option::None,
            Mode::Loading(state) => {
                ::core::option::Option::Some(
                    Group::Loading(
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
            (Mode::Idle { .. }, Mode::Idle { .. }) => true,
            (Mode::Loading(first), Mode::Loading(second)) => {
                <<door::Door as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                    door::Door,
                >>::same_state(first, second)
            }
            _ => false,
        }
    }
    fn __is_addressed(&self, event: Event) -> (bool, Event) {
        match self {
            Mode::Loading(node) => {
                match Washer::__project_into_loading(
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
        shared_storage: &mut Washer,
        event: Event,
        context: &mut Sink<'ctx, 'data>,
    ) -> ::sotu::private::Dispatched<Self, Done, Event> {
        match self {
            Mode::Loading(node) => {
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
                let visited = Washer::__project_into_loading(
                    event,
                    |event| {
                        {
                            let mut child_context = (sotu::event::unit_context)(context);
                            <door::Door as ::sotu::blocking::IntoStateMachine>::before_dispatch(
                                submachine,
                                ::sotu::StateOrSuperstate::State(&*node),
                                &event,
                                &mut child_context,
                            )
                        };
                        let ::sotu::private::Dispatched { outcome, event } = {
                            let mut child_context = (sotu::event::unit_context)(context);
                            <<door::Door as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                                door::Door,
                            >>::__dispatch(node, submachine, event, &mut child_context)
                        };
                        {
                            let mut child_context = (sotu::event::unit_context)(context);
                            <door::Door as ::sotu::blocking::IntoStateMachine>::after_dispatch(
                                submachine,
                                ::sotu::StateOrSuperstate::State(&*node),
                                &event,
                                &mut child_context,
                            )
                        };
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
                                        Mode::Loading(target),
                                    )
                                }
                                ::sotu::private::NodeOutcome::ExternalTransition {
                                    target,
                                    handler_offset,
                                } => {
                                    ::sotu::private::NodeOutcome::ExternalTransition {
                                        target: Mode::Loading(target),
                                        handler_offset,
                                    }
                                }
                                ::sotu::private::NodeOutcome::Exit(exit) => {
                                    match ::sotu::private::MountField::<
                                        door::Door,
                                    >::get_mut(&mut shared_storage.door) {
                                        ::core::option::Option::Some(submachine) => {
                                            match Washer::door_exited(submachine, exit, context) {
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
                    Washer,
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
            (Mode::Loading(source), Mode::Loading(target)) => {
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
        shared_storage: &mut Washer,
        context: &mut Sink<'ctx, 'data>,
    ) {
        match (self, target) {
            (Mode::Loading(source), Mode::Loading(target)) => {
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
                        {
                            let mut child_context = (sotu::event::unit_context)(context);
                            <door::Door as ::sotu::blocking::IntoStateMachine>::before_transition(
                                submachine,
                                source,
                                target,
                                &mut child_context,
                            )
                        };
                    }
                    ::core::option::Option::Some(deeper) => {
                        {
                            let mut child_context = (sotu::event::unit_context)(context);
                            <<door::Door as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                                door::Door,
                            >>::__before_internal_transition(
                                source,
                                target,
                                exit_levels,
                                deeper,
                                submachine,
                                &mut child_context,
                            )
                        };
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
        shared_storage: &mut Washer,
        context: &mut Sink<'ctx, 'data>,
    ) {
        match (self, target) {
            (Mode::Loading(source), Mode::Loading(target)) => {
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
                        {
                            let mut child_context = (sotu::event::unit_context)(context);
                            <door::Door as ::sotu::blocking::IntoStateMachine>::after_transition(
                                submachine,
                                source,
                                target,
                                &mut child_context,
                            )
                        };
                    }
                    ::core::option::Option::Some(deeper) => {
                        {
                            let mut child_context = (sotu::event::unit_context)(context);
                            <<door::Door as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                                door::Door,
                            >>::__after_internal_transition(
                                source,
                                target,
                                exit_levels,
                                deeper,
                                submachine,
                                &mut child_context,
                            )
                        };
                    }
                    ::core::option::Option::None => {}
                }
            }
            _ => {}
        }
    }
}
impl<'sub> ::sotu::blocking::Superstate<Washer> for Group<'sub>
where
    door::Door: ::sotu::private::BlockingSubmachine,
{
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Washer,
        event: &Event,
        _context: &mut Sink<'ctx, 'data>,
    ) -> ::sotu::Outcome<Mode, Done> {
        match self {
            Group::Loading(::sotu::SubmachineNode::Inner(_)) => ::sotu::Outcome::Super,
            Group::Loading(::sotu::SubmachineNode::Mount) => {
                match ::sotu::private::MountField::<
                    door::Door,
                >::get_mut(&mut shared_storage.door) {
                    ::core::option::Option::Some(submachine) => {
                        Washer::loading(submachine, event)
                    }
                    ::core::option::Option::None => ::sotu::Outcome::Super,
                }
            }
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Washer,
        context: &mut Sink<'ctx, 'data>,
    ) {
        match self {
            Group::Loading(::sotu::SubmachineNode::Mount) => {
                ::sotu::private::DefaultInstance::<
                    door::Door,
                >::fill_default(&mut shared_storage.door)
            }
            Group::Loading(::sotu::SubmachineNode::Inner(state)) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    door::Door,
                >::get_mut(&mut shared_storage.door) {
                    {
                        let mut child_context = (sotu::event::unit_context)(context);
                        <<door::Door as ::sotu::blocking::IntoStateMachine>::Superstate<
                            '_,
                        > as ::sotu::blocking::Superstate<
                            door::Door,
                        >>::call_entry_action(state, submachine, &mut child_context)
                    };
                }
            }
        }
    }
    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Washer,
        context: &mut Sink<'ctx, 'data>,
    ) {
        match self {
            Group::Loading(::sotu::SubmachineNode::Mount) => {
                ::sotu::private::MountField::<
                    door::Door,
                >::release(&mut shared_storage.door)
            }
            Group::Loading(::sotu::SubmachineNode::Inner(state)) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    door::Door,
                >::get_mut(&mut shared_storage.door) {
                    {
                        let mut child_context = (sotu::event::unit_context)(context);
                        <<door::Door as ::sotu::blocking::IntoStateMachine>::Superstate<
                            '_,
                        > as ::sotu::blocking::Superstate<
                            door::Door,
                        >>::call_exit_action(state, submachine, &mut child_context)
                    };
                }
            }
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Group<'_>> {
        match self {
            Group::Loading(::sotu::SubmachineNode::Mount) => ::core::option::Option::None,
            Group::Loading(::sotu::SubmachineNode::Inner(superstate)) => {
                ::core::option::Option::Some(
                    Group::Loading(
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
    fn same_state(&self, other: &Group<'_>) -> bool {
        match (self, other) {
            (
                Group::Loading(::sotu::SubmachineNode::Mount),
                Group::Loading(::sotu::SubmachineNode::Mount),
            ) => true,
            (
                Group::Loading(::sotu::SubmachineNode::Inner(first)),
                Group::Loading(::sotu::SubmachineNode::Inner(second)),
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
            Group::Loading(::sotu::SubmachineNode::Inner(node)) => {
                match Washer::__project_into_loading(
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
        shared_storage: &mut Washer,
        event: Event,
        context: &mut Sink<'ctx, 'data>,
    ) -> ::sotu::private::Dispatched<Mode, Done, Event> {
        match self {
            Group::Loading(::sotu::SubmachineNode::Inner(node)) => {
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
                let visited = Washer::__project_into_loading(
                    event,
                    |event| {
                        {
                            let mut child_context = (sotu::event::unit_context)(context);
                            <door::Door as ::sotu::blocking::IntoStateMachine>::before_dispatch(
                                submachine,
                                ::sotu::StateOrSuperstate::Superstate(&*node),
                                &event,
                                &mut child_context,
                            )
                        };
                        let ::sotu::private::Dispatched { outcome, event } = {
                            let mut child_context = (sotu::event::unit_context)(context);
                            <<door::Door as ::sotu::blocking::IntoStateMachine>::Superstate<
                                '_,
                            > as ::sotu::blocking::Superstate<
                                door::Door,
                            >>::__dispatch(node, submachine, event, &mut child_context)
                        };
                        {
                            let mut child_context = (sotu::event::unit_context)(context);
                            <door::Door as ::sotu::blocking::IntoStateMachine>::after_dispatch(
                                submachine,
                                ::sotu::StateOrSuperstate::Superstate(&*node),
                                &event,
                                &mut child_context,
                            )
                        };
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
                                        Mode::Loading(target),
                                    )
                                }
                                ::sotu::private::NodeOutcome::ExternalTransition {
                                    target,
                                    handler_offset,
                                } => {
                                    ::sotu::private::NodeOutcome::ExternalTransition {
                                        target: Mode::Loading(target),
                                        handler_offset,
                                    }
                                }
                                ::sotu::private::NodeOutcome::Exit(exit) => {
                                    match ::sotu::private::MountField::<
                                        door::Door,
                                    >::get_mut(&mut shared_storage.door) {
                                        ::core::option::Option::Some(submachine) => {
                                            match Washer::door_exited(submachine, exit, context) {
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
                    Washer,
                >::call_handler(self, shared_storage, &event, context);
                ::sotu::private::Dispatched {
                    outcome: ::sotu::private::NodeOutcome::from(outcome),
                    event: event,
                }
            }
        }
    }
}
impl ::sotu::private::Submachine for Washer {
    type Event = Event;
    type Context<'ctx, 'data: 'ctx> = Sink<'ctx, 'data>;
    type State = Mode;
    type Superstate<'sub> = Group<'sub> where Self: 'sub;
    type Exit = Done;
    fn initial() -> Mode {
        Mode::idle()
    }
}
impl ::sotu::private::BlockingSubmachine for Washer {}
impl Washer {
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
    let mut lines = Vec::new();
    let mut sink = Sink { output: &mut lines };
    let mut machine = Washer::default().state_machine();
    machine.handle(Start, &mut sink);
    machine.handle(door::Close, &mut sink);
}
