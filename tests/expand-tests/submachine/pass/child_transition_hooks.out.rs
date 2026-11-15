use sotu::prelude::*;
mod pump {
    use sotu::prelude::*;
    pub struct Pump {
        pub transitions: u32,
    }
    #[automatically_derived]
    impl ::core::default::Default for Pump {
        #[inline]
        fn default() -> Pump {
            Pump {
                transitions: ::core::default::Default::default(),
            }
        }
    }
    impl Pump {
        fn count(&mut self, _source: &State, _target: &State) {
            self.transitions += 1;
        }
        fn off() -> Outcome<State> {
            Transition(State::on())
        }
        fn on() -> Outcome<State> {
            Transition(State::off())
        }
    }
    pub enum State {
        Off {},
        On {},
    }
    impl State {
        pub fn off() -> Self {
            Self::Off {}
        }
        pub fn on() -> Self {
            Self::On {}
        }
    }
    impl ::sotu::blocking::IntoStateMachine for Pump {
        type Event = ();
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = () where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::off()
        }
        fn before_transition<'ctx, 'data: 'ctx>(
            &mut self,
            source: &State,
            target: &State,
            context: &mut (),
        ) {
            ::sotu::blocking::TransitionHook::call_transition_hook(
                &(Self::count),
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
            context: &mut (),
        ) {
            ::sotu::blocking::TransitionHook::call_transition_hook(
                &(Self::count),
                self,
                source,
                target,
                context,
            )
        }
    }
    impl ::sotu::blocking::State<Pump> for State {
        fn call_handler<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Pump,
            _event: &(),
            _context: &mut (),
        ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
            match self {
                State::Off {} => Pump::off(),
                State::On {} => Pump::on(),
            }
        }
    }
    impl ::sotu::private::Submachine for Pump {
        type Event = ();
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = () where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::off()
        }
    }
    impl ::sotu::private::BlockingSubmachine for Pump {}
    impl ::sotu::private::ExitlessSubmachine for Pump {}
}
pub struct Station {
    pump: Option<pump::Pump>,
}
#[automatically_derived]
impl ::core::default::Default for Station {
    #[inline]
    fn default() -> Station {
        Station {
            pump: ::core::default::Default::default(),
        }
    }
}
impl Station {
    fn pumping() -> Outcome<State> {
        Handled
    }
    fn closed() -> Outcome<State> {
        Handled
    }
}
pub enum State {
    Closed {},
    Pumping(<pump::Pump as ::sotu::private::Submachine>::State),
}
impl State {
    pub fn closed() -> Self {
        Self::Closed {}
    }
    pub fn pumping(state: <pump::Pump as ::sotu::private::Submachine>::State) -> Self {
        Self::Pumping(state)
    }
}
pub enum Superstate<'sub> {
    Pumping(
        ::sotu::SubmachineNode<
            <pump::Pump as ::sotu::private::Submachine>::Superstate<'sub>,
        >,
    ),
}
impl Superstate<'_> {
    pub fn pumping() -> State {
        State::Pumping(<pump::Pump as ::sotu::private::Submachine>::initial())
    }
}
impl ::sotu::blocking::IntoStateMachine for Station {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        Superstate::pumping()
    }
}
impl ::sotu::blocking::State<Station> for State
where
    pump::Pump: ::sotu::private::BlockingSubmachine,
    pump::Pump: ::sotu::private::ExitlessSubmachine,
{
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Station,
        _event: &(),
        _context: &mut (),
    ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
        match self {
            State::Closed {} => Station::closed(),
            State::Pumping(_) => ::sotu::Outcome::Super,
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Station,
        context: &mut (),
    ) {
        match self {
            State::Closed { .. } => {}
            State::Pumping(state) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    pump::Pump,
                >::get_mut(&mut shared_storage.pump) {
                    <<pump::Pump as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                        pump::Pump,
                    >>::call_entry_action(state, submachine, context);
                }
            }
        }
    }
    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Station,
        context: &mut (),
    ) {
        match self {
            State::Closed { .. } => {}
            State::Pumping(state) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    pump::Pump,
                >::get_mut(&mut shared_storage.pump) {
                    <<pump::Pump as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                        pump::Pump,
                    >>::call_exit_action(state, submachine, context);
                }
            }
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate<'_>> {
        match self {
            State::Closed {} => ::core::option::Option::None,
            State::Pumping(state) => {
                ::core::option::Option::Some(
                    Superstate::Pumping(
                        match <<pump::Pump as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            pump::Pump,
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
            (State::Closed { .. }, State::Closed { .. }) => true,
            (State::Pumping(first), State::Pumping(second)) => {
                <<pump::Pump as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                    pump::Pump,
                >>::same_state(first, second)
            }
            _ => false,
        }
    }
    fn __is_addressed(&self, event: ()) -> (bool, ()) {
        match self {
            State::Pumping(node) => {
                match Station::__project_into_pumping(
                    event,
                    |event| {
                        <<pump::Pump as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            pump::Pump,
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
        shared_storage: &mut Station,
        event: (),
        context: &mut (),
    ) -> ::sotu::private::Dispatched<Self, ::core::convert::Infallible, ()> {
        match self {
            State::Pumping(node) => {
                let mount_offset = <<pump::Pump as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::StateExt<
                    pump::Pump,
                >>::depth(node);
                let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    pump::Pump,
                >::get_mut(&mut shared_storage.pump) else {
                    return ::sotu::private::Dispatched {
                        outcome: ::sotu::private::NodeOutcome::DeferToMount {
                            mount_offset,
                        },
                        event: event,
                    };
                };
                let visited = Station::__project_into_pumping(
                    event,
                    |event| {
                        <pump::Pump as ::sotu::blocking::IntoStateMachine>::before_dispatch(
                            submachine,
                            ::sotu::StateOrSuperstate::State(&*node),
                            &event,
                            context,
                        );
                        let ::sotu::private::Dispatched { outcome, event } = <<pump::Pump as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            pump::Pump,
                        >>::__dispatch(node, submachine, event, context);
                        <pump::Pump as ::sotu::blocking::IntoStateMachine>::after_dispatch(
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
                                        State::Pumping(target),
                                    )
                                }
                                ::sotu::private::NodeOutcome::ExternalTransition {
                                    target,
                                    handler_offset,
                                } => {
                                    ::sotu::private::NodeOutcome::ExternalTransition {
                                        target: State::Pumping(target),
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
                    Station,
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
            (State::Pumping(source), State::Pumping(target)) => {
                if exit_levels
                    <= <<pump::Pump as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::StateExt<
                        pump::Pump,
                    >>::depth(source)
                {
                    1
                        + <<pump::Pump as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            pump::Pump,
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
        shared_storage: &mut Station,
        context: &mut (),
    ) {
        match (self, target) {
            (State::Pumping(source), State::Pumping(target)) => {
                if exit_levels
                    > <<pump::Pump as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::StateExt<
                        pump::Pump,
                    >>::depth(source)
                {
                    return;
                }
                let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    pump::Pump,
                >::get_mut(&mut shared_storage.pump) else {
                    return;
                };
                match nesting.checked_sub(1) {
                    ::core::option::Option::Some(0) => {
                        <pump::Pump as ::sotu::blocking::IntoStateMachine>::before_transition(
                            submachine,
                            source,
                            target,
                            context,
                        );
                    }
                    ::core::option::Option::Some(deeper) => {
                        <<pump::Pump as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            pump::Pump,
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
        shared_storage: &mut Station,
        context: &mut (),
    ) {
        match (self, target) {
            (State::Pumping(source), State::Pumping(target)) => {
                if exit_levels
                    > <<pump::Pump as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::StateExt<
                        pump::Pump,
                    >>::depth(source)
                {
                    return;
                }
                let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    pump::Pump,
                >::get_mut(&mut shared_storage.pump) else {
                    return;
                };
                match nesting.checked_sub(1) {
                    ::core::option::Option::Some(0) => {
                        <pump::Pump as ::sotu::blocking::IntoStateMachine>::after_transition(
                            submachine,
                            source,
                            target,
                            context,
                        );
                    }
                    ::core::option::Option::Some(deeper) => {
                        <<pump::Pump as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            pump::Pump,
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
impl<'sub> ::sotu::blocking::Superstate<Station> for Superstate<'sub>
where
    pump::Pump: ::sotu::private::BlockingSubmachine,
    pump::Pump: ::sotu::private::ExitlessSubmachine,
{
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Station,
        _event: &(),
        _context: &mut (),
    ) -> ::sotu::Outcome<State, ::core::convert::Infallible> {
        match self {
            Superstate::Pumping(::sotu::SubmachineNode::Inner(_)) => {
                ::sotu::Outcome::Super
            }
            Superstate::Pumping(::sotu::SubmachineNode::Mount) => Station::pumping(),
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Station,
        context: &mut (),
    ) {
        match self {
            Superstate::Pumping(::sotu::SubmachineNode::Mount) => {
                ::sotu::private::DefaultInstance::<
                    pump::Pump,
                >::fill_default(&mut shared_storage.pump)
            }
            Superstate::Pumping(::sotu::SubmachineNode::Inner(state)) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    pump::Pump,
                >::get_mut(&mut shared_storage.pump) {
                    <<pump::Pump as ::sotu::blocking::IntoStateMachine>::Superstate<
                        '_,
                    > as ::sotu::blocking::Superstate<
                        pump::Pump,
                    >>::call_entry_action(state, submachine, context);
                }
            }
        }
    }
    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Station,
        context: &mut (),
    ) {
        match self {
            Superstate::Pumping(::sotu::SubmachineNode::Mount) => {
                ::sotu::private::MountField::<
                    pump::Pump,
                >::release(&mut shared_storage.pump)
            }
            Superstate::Pumping(::sotu::SubmachineNode::Inner(state)) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    pump::Pump,
                >::get_mut(&mut shared_storage.pump) {
                    <<pump::Pump as ::sotu::blocking::IntoStateMachine>::Superstate<
                        '_,
                    > as ::sotu::blocking::Superstate<
                        pump::Pump,
                    >>::call_exit_action(state, submachine, context);
                }
            }
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate<'_>> {
        match self {
            Superstate::Pumping(::sotu::SubmachineNode::Mount) => {
                ::core::option::Option::None
            }
            Superstate::Pumping(::sotu::SubmachineNode::Inner(superstate)) => {
                ::core::option::Option::Some(
                    Superstate::Pumping(
                        match <<pump::Pump as ::sotu::blocking::IntoStateMachine>::Superstate<
                            '_,
                        > as ::sotu::blocking::Superstate<
                            pump::Pump,
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
                Superstate::Pumping(::sotu::SubmachineNode::Mount),
                Superstate::Pumping(::sotu::SubmachineNode::Mount),
            ) => true,
            (
                Superstate::Pumping(::sotu::SubmachineNode::Inner(first)),
                Superstate::Pumping(::sotu::SubmachineNode::Inner(second)),
            ) => {
                <<pump::Pump as ::sotu::blocking::IntoStateMachine>::Superstate<
                    '_,
                > as ::sotu::blocking::Superstate<pump::Pump>>::same_state(first, second)
            }
            _ => false,
        }
    }
    fn __is_addressed(&self, event: ()) -> (bool, ()) {
        match self {
            Superstate::Pumping(::sotu::SubmachineNode::Inner(node)) => {
                match Station::__project_into_pumping(
                    event,
                    |event| {
                        <<pump::Pump as ::sotu::blocking::IntoStateMachine>::Superstate<
                            '_,
                        > as ::sotu::blocking::Superstate<
                            pump::Pump,
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
        shared_storage: &mut Station,
        event: (),
        context: &mut (),
    ) -> ::sotu::private::Dispatched<State, ::core::convert::Infallible, ()> {
        match self {
            Superstate::Pumping(::sotu::SubmachineNode::Inner(node)) => {
                let mount_offset = <<pump::Pump as ::sotu::blocking::IntoStateMachine>::Superstate<
                    '_,
                > as ::sotu::blocking::SuperstateExt<pump::Pump>>::depth(node);
                let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    pump::Pump,
                >::get_mut(&mut shared_storage.pump) else {
                    return ::sotu::private::Dispatched {
                        outcome: ::sotu::private::NodeOutcome::DeferToMount {
                            mount_offset,
                        },
                        event: event,
                    };
                };
                let visited = Station::__project_into_pumping(
                    event,
                    |event| {
                        <pump::Pump as ::sotu::blocking::IntoStateMachine>::before_dispatch(
                            submachine,
                            ::sotu::StateOrSuperstate::Superstate(&*node),
                            &event,
                            context,
                        );
                        let ::sotu::private::Dispatched { outcome, event } = <<pump::Pump as ::sotu::blocking::IntoStateMachine>::Superstate<
                            '_,
                        > as ::sotu::blocking::Superstate<
                            pump::Pump,
                        >>::__dispatch(node, submachine, event, context);
                        <pump::Pump as ::sotu::blocking::IntoStateMachine>::after_dispatch(
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
                                        State::Pumping(target),
                                    )
                                }
                                ::sotu::private::NodeOutcome::ExternalTransition {
                                    target,
                                    handler_offset,
                                } => {
                                    ::sotu::private::NodeOutcome::ExternalTransition {
                                        target: State::Pumping(target),
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
                    Station,
                >::call_handler(self, shared_storage, &event, context);
                ::sotu::private::Dispatched {
                    outcome: ::sotu::private::NodeOutcome::from(outcome),
                    event: event,
                }
            }
        }
    }
}
impl ::sotu::private::Submachine for Station {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        Superstate::pumping()
    }
}
impl ::sotu::private::BlockingSubmachine for Station {}
impl ::sotu::private::ExitlessSubmachine for Station {}
impl Station {
    #[doc(hidden)]
    #[allow(unreachable_code)]
    fn __project_into_pumping<Visited>(
        event: (),
        visit: impl ::core::ops::FnOnce(
            <pump::Pump as ::sotu::private::Submachine>::Event,
        ) -> (Visited, <pump::Pump as ::sotu::private::Submachine>::Event),
    ) -> ::core::result::Result<(Visited, ()), ()> {
        if !<<pump::Pump as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
            (),
        >>::accepts(&event) {
            return ::core::result::Result::Err(event);
        }
        match <<pump::Pump as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
            (),
        >>::project(event) {
            ::core::result::Result::Ok(child_event) => {
                let (visited, child_event) = visit(child_event);
                ::core::result::Result::Ok((
                    visited,
                    <<pump::Pump as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
                        (),
                    >>::embed(child_event),
                ))
            }
            ::core::result::Result::Err(event) => ::core::result::Result::Err(event),
        }
    }
}
fn main() {
    let mut machine = Station::default().state_machine();
    machine.step(&mut ());
}
