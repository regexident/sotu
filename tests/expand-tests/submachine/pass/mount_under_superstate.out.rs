use sotu::prelude::*;
mod fan {
    use sotu::prelude::*;
    pub struct Fan;
    #[automatically_derived]
    impl ::core::default::Default for Fan {
        #[inline]
        fn default() -> Fan {
            Fan {}
        }
    }
    impl Fan {
        fn slow() -> Outcome<State> {
            Super
        }
        fn fast() -> Outcome<State> {
            Super
        }
        fn spinning() -> Outcome<State> {
            Handled
        }
    }
    pub enum State {
        Slow {},
        Fast {},
    }
    impl State {
        pub fn slow() -> Self {
            Self::Slow {}
        }
        pub fn fast() -> Self {
            Self::Fast {}
        }
    }
    pub enum Superstate {
        Spinning {},
    }
    impl ::sotu::blocking::IntoStateMachine for Fan {
        type Event = ();
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = Superstate where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::slow()
        }
    }
    impl ::sotu::blocking::State<Fan> for State {
        fn call_handler<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Fan,
            _event: &(),
            _context: &mut (),
        ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
            match self {
                State::Slow {} => Fan::slow(),
                State::Fast {} => Fan::fast(),
            }
        }
        fn superstate(&mut self) -> ::core::option::Option<Superstate> {
            match self {
                State::Slow {} => ::core::option::Option::Some(Superstate::Spinning {}),
                State::Fast {} => ::core::option::Option::Some(Superstate::Spinning {}),
            }
        }
    }
    impl ::sotu::blocking::Superstate<Fan> for Superstate {
        fn call_handler<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Fan,
            _event: &(),
            _context: &mut (),
        ) -> ::sotu::Outcome<State, ::core::convert::Infallible> {
            match self {
                Superstate::Spinning {} => Fan::spinning(),
            }
        }
        fn superstate(&mut self) -> ::core::option::Option<Superstate> {
            match self {
                Superstate::Spinning {} => ::core::option::Option::None,
            }
        }
        fn same_state(&self, other: &Superstate) -> bool {
            match (self, other) {
                (Superstate::Spinning { .. }, Superstate::Spinning { .. }) => true,
            }
        }
    }
    impl ::sotu::private::Submachine for Fan {
        type Event = ();
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = Superstate where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::slow()
        }
    }
    impl ::sotu::private::BlockingSubmachine for Fan {}
    impl ::sotu::private::ExitlessSubmachine for Fan {}
}
pub struct Room {
    fan: fan::Fan,
}
#[automatically_derived]
impl ::core::default::Default for Room {
    #[inline]
    fn default() -> Room {
        Room {
            fan: ::core::default::Default::default(),
        }
    }
}
impl Room {
    fn lit() -> Outcome<State> {
        Super
    }
    /// The fan keeps its storage between activations.
    fn cool() -> Outcome<State> {
        Transition(State::cooling(fan::State::fast()))
    }
    fn powered() -> Outcome<State> {
        Transition(State::lit())
    }
}
pub enum State {
    Lit {},
    /// The fan keeps its storage between activations.
    Cooling(<fan::Fan as ::sotu::private::Submachine>::State),
}
impl State {
    pub fn lit() -> Self {
        Self::Lit {}
    }
    pub fn cooling(state: <fan::Fan as ::sotu::private::Submachine>::State) -> Self {
        Self::Cooling(state)
    }
}
pub enum Superstate<'sub> {
    Powered {},
    /// The fan keeps its storage between activations.
    Cooling(
        ::sotu::SubmachineNode<
            <fan::Fan as ::sotu::private::Submachine>::Superstate<'sub>,
        >,
    ),
}
impl Superstate<'_> {
    pub fn cooling() -> State {
        State::Cooling(<fan::Fan as ::sotu::private::Submachine>::initial())
    }
}
impl ::sotu::blocking::IntoStateMachine for Room {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::cooling(fan::State::slow())
    }
}
impl ::sotu::blocking::State<Room> for State
where
    fan::Fan: ::sotu::private::BlockingSubmachine,
    fan::Fan: ::sotu::private::ExitlessSubmachine,
{
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Room,
        _event: &(),
        _context: &mut (),
    ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
        match self {
            State::Lit {} => Room::lit(),
            State::Cooling(_) => ::sotu::Outcome::Super,
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Room,
        context: &mut (),
    ) {
        match self {
            State::Lit { .. } => {}
            State::Cooling(state) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    fan::Fan,
                >::get_mut(&mut shared_storage.fan) {
                    <<fan::Fan as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                        fan::Fan,
                    >>::call_entry_action(state, submachine, context);
                }
            }
        }
    }
    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Room,
        context: &mut (),
    ) {
        match self {
            State::Lit { .. } => {}
            State::Cooling(state) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    fan::Fan,
                >::get_mut(&mut shared_storage.fan) {
                    <<fan::Fan as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                        fan::Fan,
                    >>::call_exit_action(state, submachine, context);
                }
            }
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate<'_>> {
        match self {
            State::Lit {} => ::core::option::Option::Some(Superstate::Powered {}),
            State::Cooling(state) => {
                ::core::option::Option::Some(
                    Superstate::Cooling(
                        match <<fan::Fan as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            fan::Fan,
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
            (State::Lit { .. }, State::Lit { .. }) => true,
            (State::Cooling(first), State::Cooling(second)) => {
                <<fan::Fan as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                    fan::Fan,
                >>::same_state(first, second)
            }
            _ => false,
        }
    }
    fn __is_addressed(&self, event: ()) -> (bool, ()) {
        match self {
            State::Cooling(node) => {
                match Room::__project_into_cooling(
                    event,
                    |event| {
                        <<fan::Fan as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            fan::Fan,
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
        shared_storage: &mut Room,
        event: (),
        context: &mut (),
    ) -> ::sotu::private::Dispatched<Self, ::core::convert::Infallible, ()> {
        match self {
            State::Cooling(node) => {
                let mount_offset = <<fan::Fan as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::StateExt<
                    fan::Fan,
                >>::depth(node);
                let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    fan::Fan,
                >::get_mut(&mut shared_storage.fan) else {
                    return ::sotu::private::Dispatched {
                        outcome: ::sotu::private::NodeOutcome::DeferToMount {
                            mount_offset,
                        },
                        event: event,
                    };
                };
                let visited = Room::__project_into_cooling(
                    event,
                    |event| {
                        <fan::Fan as ::sotu::blocking::IntoStateMachine>::before_dispatch(
                            submachine,
                            ::sotu::StateOrSuperstate::State(&*node),
                            &event,
                            context,
                        );
                        let ::sotu::private::Dispatched { outcome, event } = <<fan::Fan as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            fan::Fan,
                        >>::__dispatch(node, submachine, event, context);
                        <fan::Fan as ::sotu::blocking::IntoStateMachine>::after_dispatch(
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
                                        State::Cooling(target),
                                    )
                                }
                                ::sotu::private::NodeOutcome::ExternalTransition {
                                    target,
                                    handler_offset,
                                } => {
                                    ::sotu::private::NodeOutcome::ExternalTransition {
                                        target: State::Cooling(target),
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
                    Room,
                >::call_handler(self, shared_storage, &event, context);
                ::sotu::private::Dispatched {
                    outcome: ::sotu::private::NodeOutcome::from(outcome),
                    event: event,
                }
            }
        }
    }
}
impl<'sub> ::sotu::blocking::Superstate<Room> for Superstate<'sub>
where
    fan::Fan: ::sotu::private::BlockingSubmachine,
    fan::Fan: ::sotu::private::ExitlessSubmachine,
{
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Room,
        _event: &(),
        _context: &mut (),
    ) -> ::sotu::Outcome<State, ::core::convert::Infallible> {
        match self {
            Superstate::Powered {} => Room::powered(),
            Superstate::Cooling(::sotu::SubmachineNode::Inner(_)) => {
                ::sotu::Outcome::Super
            }
            Superstate::Cooling(::sotu::SubmachineNode::Mount) => Room::cool(),
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Room,
        context: &mut (),
    ) {
        match self {
            Superstate::Powered { .. } => {}
            Superstate::Cooling(::sotu::SubmachineNode::Mount) => {
                ::sotu::private::DefaultInstance::<
                    fan::Fan,
                >::fill_default(&mut shared_storage.fan)
            }
            Superstate::Cooling(::sotu::SubmachineNode::Inner(state)) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    fan::Fan,
                >::get_mut(&mut shared_storage.fan) {
                    <<fan::Fan as ::sotu::blocking::IntoStateMachine>::Superstate<
                        '_,
                    > as ::sotu::blocking::Superstate<
                        fan::Fan,
                    >>::call_entry_action(state, submachine, context);
                }
            }
        }
    }
    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Room,
        context: &mut (),
    ) {
        match self {
            Superstate::Powered { .. } => {}
            Superstate::Cooling(::sotu::SubmachineNode::Mount) => {
                ::sotu::private::MountField::<fan::Fan>::release(&mut shared_storage.fan)
            }
            Superstate::Cooling(::sotu::SubmachineNode::Inner(state)) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    fan::Fan,
                >::get_mut(&mut shared_storage.fan) {
                    <<fan::Fan as ::sotu::blocking::IntoStateMachine>::Superstate<
                        '_,
                    > as ::sotu::blocking::Superstate<
                        fan::Fan,
                    >>::call_exit_action(state, submachine, context);
                }
            }
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate<'_>> {
        match self {
            Superstate::Powered {} => ::core::option::Option::None,
            Superstate::Cooling(::sotu::SubmachineNode::Mount) => {
                ::core::option::Option::Some(Superstate::Powered {})
            }
            Superstate::Cooling(::sotu::SubmachineNode::Inner(superstate)) => {
                ::core::option::Option::Some(
                    Superstate::Cooling(
                        match <<fan::Fan as ::sotu::blocking::IntoStateMachine>::Superstate<
                            '_,
                        > as ::sotu::blocking::Superstate<
                            fan::Fan,
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
            (Superstate::Powered { .. }, Superstate::Powered { .. }) => true,
            (
                Superstate::Cooling(::sotu::SubmachineNode::Mount),
                Superstate::Cooling(::sotu::SubmachineNode::Mount),
            ) => true,
            (
                Superstate::Cooling(::sotu::SubmachineNode::Inner(first)),
                Superstate::Cooling(::sotu::SubmachineNode::Inner(second)),
            ) => {
                <<fan::Fan as ::sotu::blocking::IntoStateMachine>::Superstate<
                    '_,
                > as ::sotu::blocking::Superstate<fan::Fan>>::same_state(first, second)
            }
            _ => false,
        }
    }
    fn __is_addressed(&self, event: ()) -> (bool, ()) {
        match self {
            Superstate::Cooling(::sotu::SubmachineNode::Inner(node)) => {
                match Room::__project_into_cooling(
                    event,
                    |event| {
                        <<fan::Fan as ::sotu::blocking::IntoStateMachine>::Superstate<
                            '_,
                        > as ::sotu::blocking::Superstate<
                            fan::Fan,
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
        shared_storage: &mut Room,
        event: (),
        context: &mut (),
    ) -> ::sotu::private::Dispatched<State, ::core::convert::Infallible, ()> {
        match self {
            Superstate::Cooling(::sotu::SubmachineNode::Inner(node)) => {
                let mount_offset = <<fan::Fan as ::sotu::blocking::IntoStateMachine>::Superstate<
                    '_,
                > as ::sotu::blocking::SuperstateExt<fan::Fan>>::depth(node);
                let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    fan::Fan,
                >::get_mut(&mut shared_storage.fan) else {
                    return ::sotu::private::Dispatched {
                        outcome: ::sotu::private::NodeOutcome::DeferToMount {
                            mount_offset,
                        },
                        event: event,
                    };
                };
                let visited = Room::__project_into_cooling(
                    event,
                    |event| {
                        <fan::Fan as ::sotu::blocking::IntoStateMachine>::before_dispatch(
                            submachine,
                            ::sotu::StateOrSuperstate::Superstate(&*node),
                            &event,
                            context,
                        );
                        let ::sotu::private::Dispatched { outcome, event } = <<fan::Fan as ::sotu::blocking::IntoStateMachine>::Superstate<
                            '_,
                        > as ::sotu::blocking::Superstate<
                            fan::Fan,
                        >>::__dispatch(node, submachine, event, context);
                        <fan::Fan as ::sotu::blocking::IntoStateMachine>::after_dispatch(
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
                                        State::Cooling(target),
                                    )
                                }
                                ::sotu::private::NodeOutcome::ExternalTransition {
                                    target,
                                    handler_offset,
                                } => {
                                    ::sotu::private::NodeOutcome::ExternalTransition {
                                        target: State::Cooling(target),
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
                    Room,
                >::call_handler(self, shared_storage, &event, context);
                ::sotu::private::Dispatched {
                    outcome: ::sotu::private::NodeOutcome::from(outcome),
                    event: event,
                }
            }
        }
    }
}
impl ::sotu::private::Submachine for Room {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::cooling(fan::State::slow())
    }
}
impl ::sotu::private::BlockingSubmachine for Room {}
impl ::sotu::private::ExitlessSubmachine for Room {}
impl Room {
    #[doc(hidden)]
    #[allow(unreachable_code)]
    fn __project_into_cooling<Visited>(
        event: (),
        visit: impl ::core::ops::FnOnce(
            <fan::Fan as ::sotu::private::Submachine>::Event,
        ) -> (Visited, <fan::Fan as ::sotu::private::Submachine>::Event),
    ) -> ::core::result::Result<(Visited, ()), ()> {
        if !<<fan::Fan as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
            (),
        >>::accepts(&event) {
            return ::core::result::Result::Err(event);
        }
        match <<fan::Fan as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
            (),
        >>::project(event) {
            ::core::result::Result::Ok(child_event) => {
                let (visited, child_event) = visit(child_event);
                ::core::result::Result::Ok((
                    visited,
                    <<fan::Fan as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
                        (),
                    >>::embed(child_event),
                ))
            }
            ::core::result::Result::Err(event) => ::core::result::Result::Err(event),
        }
    }
}
fn main() {
    let mut machine = Room::default().state_machine();
    machine.step(&mut ());
}
