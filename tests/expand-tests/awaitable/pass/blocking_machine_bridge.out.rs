use sotu::prelude::*;
mod door {
    use sotu::prelude::*;
    pub struct Door;
    #[automatically_derived]
    impl ::core::default::Default for Door {
        #[inline]
        fn default() -> Door {
            Door {}
        }
    }
    pub enum Event {}
    impl<SotuParentEvent> ::sotu::event::SubEvent<SotuParentEvent> for Event {
        fn accepts(_parent: &SotuParentEvent) -> bool {
            false
        }
        fn project(
            parent: SotuParentEvent,
        ) -> ::core::result::Result<Self, SotuParentEvent> {
            ::core::result::Result::Err(parent)
        }
        fn embed(self) -> SotuParentEvent {
            match self {}
        }
    }
    impl Door {
        fn opened(event: &Event) -> Outcome<State> {
            match *event {}
        }
        fn frame(event: &Event) -> Outcome<State> {
            match *event {}
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
        type Exit = ::core::convert::Infallible;
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
        ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
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
        ) -> ::sotu::Outcome<State, ::core::convert::Infallible> {
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
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::opened()
        }
    }
    impl ::sotu::private::BlockingSubmachine for Door {}
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
    }
}
pub struct Hall {
    pub door: Option<door::Door>,
    pub visits: u32,
}
#[automatically_derived]
impl ::core::default::Default for Hall {
    #[inline]
    fn default() -> Hall {
        Hall {
            door: ::core::default::Default::default(),
            visits: ::core::default::Default::default(),
        }
    }
}
impl Hall {
    fn idle(visits: &mut u32) -> Outcome<State> {
        *visits += 1;
        Super
    }
    fn loading() -> Outcome<State> {
        Handled
    }
    fn lobby(visits: &u32) -> Outcome<State> {
        if *visits > 1 { Handled } else { Super }
    }
    fn region() -> Outcome<State> {
        Handled
    }
}
pub enum State {
    Idle { visits: u32 },
    Loading(<door::Door as ::sotu::private::Submachine>::State),
}
impl State {
    pub fn idle(visits: u32) -> Self {
        Self::Idle { visits }
    }
    pub fn loading(state: <door::Door as ::sotu::private::Submachine>::State) -> Self {
        Self::Loading(state)
    }
}
pub enum Superstate<'sub> {
    Lobby { visits: &'sub u32 },
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
impl ::sotu::blocking::IntoStateMachine for Hall {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::idle(0)
    }
}
impl ::sotu::blocking::State<Hall> for State
where
    door::Door: ::sotu::private::BlockingSubmachine,
    door::Door: ::sotu::private::ExitlessSubmachine,
{
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Hall,
        _event: &(),
        _context: &mut (),
    ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
        match self {
            State::Idle { visits } => Hall::idle(visits),
            State::Loading(_) => ::sotu::Outcome::Super,
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Hall,
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
        shared_storage: &mut Hall,
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
            State::Idle { visits } => {
                ::core::option::Option::Some(Superstate::Lobby { visits })
            }
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
    fn __is_addressed(&self, event: ()) -> (bool, ()) {
        match self {
            State::Loading(node) => {
                match Hall::__project_into_loading(
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
        shared_storage: &mut Hall,
        event: (),
        context: &mut (),
    ) -> ::sotu::private::Dispatched<Self, ::core::convert::Infallible, ()> {
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
                let visited = Hall::__project_into_loading(
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
                let outcome = ::sotu::blocking::State::<
                    Hall,
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
        shared_storage: &mut Hall,
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
        shared_storage: &mut Hall,
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
impl<'sub> ::sotu::blocking::Superstate<Hall> for Superstate<'sub>
where
    door::Door: ::sotu::private::BlockingSubmachine,
    door::Door: ::sotu::private::ExitlessSubmachine,
{
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Hall,
        _event: &(),
        _context: &mut (),
    ) -> ::sotu::Outcome<State, ::core::convert::Infallible> {
        match self {
            Superstate::Lobby { visits } => Hall::lobby(visits),
            Superstate::Region {} => Hall::region(),
            Superstate::Loading(::sotu::SubmachineNode::Inner(_)) => {
                ::sotu::Outcome::Super
            }
            Superstate::Loading(::sotu::SubmachineNode::Mount) => Hall::loading(),
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Hall,
        context: &mut (),
    ) {
        match self {
            Superstate::Lobby { .. } => {}
            Superstate::Region { .. } => {}
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
        shared_storage: &mut Hall,
        context: &mut (),
    ) {
        match self {
            Superstate::Lobby { .. } => {}
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
            Superstate::Lobby { .. } => {
                ::core::option::Option::Some(Superstate::Region {})
            }
            Superstate::Region {} => ::core::option::Option::None,
            Superstate::Loading(::sotu::SubmachineNode::Mount) => {
                ::core::option::Option::Some(Superstate::Region {})
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
            (Superstate::Lobby { .. }, Superstate::Lobby { .. }) => true,
            (Superstate::Region { .. }, Superstate::Region { .. }) => true,
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
    fn __is_addressed(&self, event: ()) -> (bool, ()) {
        match self {
            Superstate::Loading(::sotu::SubmachineNode::Inner(node)) => {
                match Hall::__project_into_loading(
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
        shared_storage: &mut Hall,
        event: (),
        context: &mut (),
    ) -> ::sotu::private::Dispatched<State, ::core::convert::Infallible, ()> {
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
                let visited = Hall::__project_into_loading(
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
                let outcome = ::sotu::blocking::Superstate::<
                    Hall,
                >::call_handler(self, shared_storage, &event, context);
                ::sotu::private::Dispatched {
                    outcome: ::sotu::private::NodeOutcome::from(outcome),
                    event: event,
                }
            }
        }
    }
}
impl ::sotu::private::Submachine for Hall {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::idle(0)
    }
}
impl ::sotu::private::BlockingSubmachine for Hall {}
impl ::sotu::private::ExitlessSubmachine for Hall {}
impl ::sotu::private::AwaitableSubmachine for Hall
where
    door::Door: ::sotu::private::BlockingSubmachine,
    door::Door: ::sotu::private::ExitlessSubmachine,
    door::Door: ::sotu::private::AwaitableSubmachine,
{
    fn state_superstate<'sub>(
        state: &'sub mut <Self as ::sotu::private::Submachine>::State,
    ) -> ::core::option::Option<<Self as ::sotu::private::Submachine>::Superstate<'sub>>
    where
        Self: 'sub,
    {
        <<Hall as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
            Hall,
        >>::superstate(state)
    }
    fn state_same(
        first: &<Self as ::sotu::private::Submachine>::State,
        second: &<Self as ::sotu::private::Submachine>::State,
    ) -> bool {
        <<Hall as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
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
        <<Hall as ::sotu::blocking::IntoStateMachine>::Superstate<
            '_,
        > as ::sotu::blocking::Superstate<Hall>>::same_state(first, second)
    }
    fn state_depth(state: &mut <Self as ::sotu::private::Submachine>::State) -> usize {
        <<Hall as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::StateExt<
            Hall,
        >>::depth(state)
    }
    fn state_is_addressed(
        state: &<Self as ::sotu::private::Submachine>::State,
        event: <Self as ::sotu::private::Submachine>::Event,
    ) -> (bool, <Self as ::sotu::private::Submachine>::Event) {
        <<Hall as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
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
        <<Hall as ::sotu::blocking::IntoStateMachine>::Superstate<
            '_,
        > as ::sotu::blocking::Superstate<Hall>>::__is_addressed(superstate, event)
    }
}
impl Hall {
    #[doc(hidden)]
    #[allow(unreachable_code)]
    fn __project_into_loading<Visited>(
        event: (),
        visit: impl ::core::ops::FnOnce(
            <door::Door as ::sotu::private::Submachine>::Event,
        ) -> (Visited, <door::Door as ::sotu::private::Submachine>::Event),
    ) -> ::core::result::Result<(Visited, ()), ()> {
        if !<<door::Door as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
            (),
        >>::accepts(&event) {
            return ::core::result::Result::Err(event);
        }
        match <<door::Door as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
            (),
        >>::project(event) {
            ::core::result::Result::Ok(child_event) => {
                let (visited, child_event) = visit(child_event);
                ::core::result::Result::Ok((
                    visited,
                    <<door::Door as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
                        (),
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
