use sotu::prelude::*;
mod log {
    use sotu::prelude::*;
    /// The log has no `Default`, since a persistent field is never created
    /// by the machine.
    pub struct Log {
        pub entries: u32,
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
    impl Log {
        fn writing(event: &Event) -> Outcome<State> {
            match *event {}
        }
        fn count(&mut self) {
            self.entries += 1;
        }
    }
    pub enum State {
        Writing {},
    }
    impl State {
        pub fn writing() -> Self {
            Self::Writing {}
        }
    }
    impl ::sotu::blocking::IntoStateMachine for Log {
        type Event = Event;
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = () where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::writing()
        }
    }
    impl ::sotu::blocking::State<Log> for State {
        fn call_handler<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Log,
            event: &Event,
            _context: &mut (),
        ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
            match self {
                State::Writing {} => Log::writing(event),
            }
        }
        fn call_entry_action<'ctx, 'data: 'ctx>(
            &mut self,
            shared_storage: &mut Log,
            _context: &mut (),
        ) {
            match self {
                State::Writing {} => Log::count(shared_storage),
            }
        }
    }
    impl ::sotu::private::Submachine for Log {
        type Event = Event;
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = () where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::writing()
        }
    }
    impl ::sotu::private::BlockingSubmachine for Log {}
    impl ::sotu::private::ExitlessSubmachine for Log {}
}
pub enum Event {
    Enter,
    Leave,
}
pub struct House {
    log: log::Log,
}
impl House {
    fn idle(event: &Event) -> Outcome<State> {
        match event {
            Event::Enter => Transition(Superstate::logging()),
            Event::Leave => Handled,
        }
    }
    fn logging(event: &Event) -> Outcome<State> {
        match event {
            Event::Enter => Handled,
            Event::Leave => Transition(State::idle()),
        }
    }
}
pub enum State {
    Idle {},
    Logging(<log::Log as ::sotu::private::Submachine>::State),
}
impl State {
    pub fn idle() -> Self {
        Self::Idle {}
    }
    pub fn logging(state: <log::Log as ::sotu::private::Submachine>::State) -> Self {
        Self::Logging(state)
    }
}
pub enum Superstate<'sub> {
    Logging(
        ::sotu::SubmachineNode<
            <log::Log as ::sotu::private::Submachine>::Superstate<'sub>,
        >,
    ),
}
impl Superstate<'_> {
    pub fn logging() -> State {
        State::Logging(<log::Log as ::sotu::private::Submachine>::initial())
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
    log::Log: ::sotu::private::BlockingSubmachine,
    log::Log: ::sotu::private::ExitlessSubmachine,
{
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut House,
        event: &Event,
        _context: &mut (),
    ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
        match self {
            State::Idle {} => House::idle(event),
            State::Logging(_) => ::sotu::Outcome::Super,
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut House,
        context: &mut (),
    ) {
        match self {
            State::Idle { .. } => {}
            State::Logging(state) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    log::Log,
                >::get_mut(&mut shared_storage.log) {
                    <<log::Log as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                        log::Log,
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
            State::Logging(state) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    log::Log,
                >::get_mut(&mut shared_storage.log) {
                    <<log::Log as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                        log::Log,
                    >>::call_exit_action(state, submachine, context);
                }
            }
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate<'_>> {
        match self {
            State::Idle {} => ::core::option::Option::None,
            State::Logging(state) => {
                ::core::option::Option::Some(
                    Superstate::Logging(
                        match <<log::Log as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            log::Log,
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
            (State::Logging(first), State::Logging(second)) => {
                <<log::Log as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                    log::Log,
                >>::same_state(first, second)
            }
            _ => false,
        }
    }
    fn __is_addressed(&self, event: Event) -> (bool, Event) {
        match self {
            State::Logging(node) => {
                match House::__project_into_logging(
                    event,
                    |event| {
                        <<log::Log as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            log::Log,
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
            State::Logging(node) => {
                let mount_offset = <<log::Log as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::StateExt<
                    log::Log,
                >>::depth(node);
                let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    log::Log,
                >::get_mut(&mut shared_storage.log) else {
                    return ::sotu::private::Dispatched {
                        outcome: ::sotu::private::NodeOutcome::DeferToMount {
                            mount_offset,
                        },
                        event: event,
                    };
                };
                let visited = House::__project_into_logging(
                    event,
                    |event| {
                        <log::Log as ::sotu::blocking::IntoStateMachine>::before_dispatch(
                            submachine,
                            ::sotu::StateOrSuperstate::State(&*node),
                            &event,
                            context,
                        );
                        let ::sotu::private::Dispatched { outcome, event } = <<log::Log as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            log::Log,
                        >>::__dispatch(node, submachine, event, context);
                        <log::Log as ::sotu::blocking::IntoStateMachine>::after_dispatch(
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
                                        State::Logging(target),
                                    )
                                }
                                ::sotu::private::NodeOutcome::ExternalTransition {
                                    target,
                                    handler_offset,
                                } => {
                                    ::sotu::private::NodeOutcome::ExternalTransition {
                                        target: State::Logging(target),
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
            (State::Logging(source), State::Logging(target)) => {
                if exit_levels
                    <= <<log::Log as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::StateExt<
                        log::Log,
                    >>::depth(source)
                {
                    1
                        + <<log::Log as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            log::Log,
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
            (State::Logging(source), State::Logging(target)) => {
                if exit_levels
                    > <<log::Log as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::StateExt<
                        log::Log,
                    >>::depth(source)
                {
                    return;
                }
                let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    log::Log,
                >::get_mut(&mut shared_storage.log) else {
                    return;
                };
                match nesting.checked_sub(1) {
                    ::core::option::Option::Some(0) => {
                        <log::Log as ::sotu::blocking::IntoStateMachine>::before_transition(
                            submachine,
                            source,
                            target,
                            context,
                        );
                    }
                    ::core::option::Option::Some(deeper) => {
                        <<log::Log as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            log::Log,
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
            (State::Logging(source), State::Logging(target)) => {
                if exit_levels
                    > <<log::Log as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::StateExt<
                        log::Log,
                    >>::depth(source)
                {
                    return;
                }
                let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    log::Log,
                >::get_mut(&mut shared_storage.log) else {
                    return;
                };
                match nesting.checked_sub(1) {
                    ::core::option::Option::Some(0) => {
                        <log::Log as ::sotu::blocking::IntoStateMachine>::after_transition(
                            submachine,
                            source,
                            target,
                            context,
                        );
                    }
                    ::core::option::Option::Some(deeper) => {
                        <<log::Log as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            log::Log,
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
    log::Log: ::sotu::private::BlockingSubmachine,
    log::Log: ::sotu::private::ExitlessSubmachine,
{
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut House,
        event: &Event,
        _context: &mut (),
    ) -> ::sotu::Outcome<State, ::core::convert::Infallible> {
        match self {
            Superstate::Logging(::sotu::SubmachineNode::Inner(_)) => {
                ::sotu::Outcome::Super
            }
            Superstate::Logging(::sotu::SubmachineNode::Mount) => House::logging(event),
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut House,
        context: &mut (),
    ) {
        match self {
            Superstate::Logging(::sotu::SubmachineNode::Mount) => {
                ::sotu::private::DefaultInstance::<
                    log::Log,
                >::fill_default(&mut shared_storage.log)
            }
            Superstate::Logging(::sotu::SubmachineNode::Inner(state)) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    log::Log,
                >::get_mut(&mut shared_storage.log) {
                    <<log::Log as ::sotu::blocking::IntoStateMachine>::Superstate<
                        '_,
                    > as ::sotu::blocking::Superstate<
                        log::Log,
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
            Superstate::Logging(::sotu::SubmachineNode::Mount) => {
                ::sotu::private::MountField::<log::Log>::release(&mut shared_storage.log)
            }
            Superstate::Logging(::sotu::SubmachineNode::Inner(state)) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    log::Log,
                >::get_mut(&mut shared_storage.log) {
                    <<log::Log as ::sotu::blocking::IntoStateMachine>::Superstate<
                        '_,
                    > as ::sotu::blocking::Superstate<
                        log::Log,
                    >>::call_exit_action(state, submachine, context);
                }
            }
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate<'_>> {
        match self {
            Superstate::Logging(::sotu::SubmachineNode::Mount) => {
                ::core::option::Option::None
            }
            Superstate::Logging(::sotu::SubmachineNode::Inner(superstate)) => {
                ::core::option::Option::Some(
                    Superstate::Logging(
                        match <<log::Log as ::sotu::blocking::IntoStateMachine>::Superstate<
                            '_,
                        > as ::sotu::blocking::Superstate<
                            log::Log,
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
                Superstate::Logging(::sotu::SubmachineNode::Mount),
                Superstate::Logging(::sotu::SubmachineNode::Mount),
            ) => true,
            (
                Superstate::Logging(::sotu::SubmachineNode::Inner(first)),
                Superstate::Logging(::sotu::SubmachineNode::Inner(second)),
            ) => {
                <<log::Log as ::sotu::blocking::IntoStateMachine>::Superstate<
                    '_,
                > as ::sotu::blocking::Superstate<log::Log>>::same_state(first, second)
            }
            _ => false,
        }
    }
    fn __is_addressed(&self, event: Event) -> (bool, Event) {
        match self {
            Superstate::Logging(::sotu::SubmachineNode::Inner(node)) => {
                match House::__project_into_logging(
                    event,
                    |event| {
                        <<log::Log as ::sotu::blocking::IntoStateMachine>::Superstate<
                            '_,
                        > as ::sotu::blocking::Superstate<
                            log::Log,
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
            Superstate::Logging(::sotu::SubmachineNode::Inner(node)) => {
                let mount_offset = <<log::Log as ::sotu::blocking::IntoStateMachine>::Superstate<
                    '_,
                > as ::sotu::blocking::SuperstateExt<log::Log>>::depth(node);
                let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    log::Log,
                >::get_mut(&mut shared_storage.log) else {
                    return ::sotu::private::Dispatched {
                        outcome: ::sotu::private::NodeOutcome::DeferToMount {
                            mount_offset,
                        },
                        event: event,
                    };
                };
                let visited = House::__project_into_logging(
                    event,
                    |event| {
                        <log::Log as ::sotu::blocking::IntoStateMachine>::before_dispatch(
                            submachine,
                            ::sotu::StateOrSuperstate::Superstate(&*node),
                            &event,
                            context,
                        );
                        let ::sotu::private::Dispatched { outcome, event } = <<log::Log as ::sotu::blocking::IntoStateMachine>::Superstate<
                            '_,
                        > as ::sotu::blocking::Superstate<
                            log::Log,
                        >>::__dispatch(node, submachine, event, context);
                        <log::Log as ::sotu::blocking::IntoStateMachine>::after_dispatch(
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
                                        State::Logging(target),
                                    )
                                }
                                ::sotu::private::NodeOutcome::ExternalTransition {
                                    target,
                                    handler_offset,
                                } => {
                                    ::sotu::private::NodeOutcome::ExternalTransition {
                                        target: State::Logging(target),
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
    fn __project_into_logging<Visited>(
        event: Event,
        visit: impl ::core::ops::FnOnce(
            <log::Log as ::sotu::private::Submachine>::Event,
        ) -> (Visited, <log::Log as ::sotu::private::Submachine>::Event),
    ) -> ::core::result::Result<(Visited, Event), Event> {
        if !<<log::Log as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
            Event,
        >>::accepts(&event) {
            return ::core::result::Result::Err(event);
        }
        match <<log::Log as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
            Event,
        >>::project(event) {
            ::core::result::Result::Ok(child_event) => {
                let (visited, child_event) = visit(child_event);
                ::core::result::Result::Ok((
                    visited,
                    <<log::Log as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
                        Event,
                    >>::embed(child_event),
                ))
            }
            ::core::result::Result::Err(event) => ::core::result::Result::Err(event),
        }
    }
}
fn main() {
    let mut machine = House {
        log: log::Log { entries: 0 },
    }
        .state_machine();
    machine.handle(Event::Enter, &mut ());
    machine.handle(Event::Leave, &mut ());
}
