use sotu::prelude::*;
mod lamp {
    use sotu::prelude::*;
    pub struct Lamp {
        pub dispatches: u32,
    }
    #[automatically_derived]
    impl ::core::default::Default for Lamp {
        #[inline]
        fn default() -> Lamp {
            Lamp {
                dispatches: ::core::default::Default::default(),
            }
        }
    }
    impl Lamp {
        fn count(
            &mut self,
            _node: StateOrSuperstate<'_, State, Superstate>,
            _event: &(),
        ) {
            self.dispatches += 1;
        }
        fn dim() -> Outcome<State> {
            Super
        }
        fn glowing() -> Outcome<State> {
            Handled
        }
    }
    pub enum State {
        Dim {},
    }
    impl State {
        pub fn dim() -> Self {
            Self::Dim {}
        }
    }
    pub enum Superstate {
        Glowing {},
    }
    impl ::sotu::blocking::IntoStateMachine for Lamp {
        type Event = ();
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = Superstate where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::dim()
        }
        fn before_dispatch<'ctx, 'data: 'ctx>(
            &mut self,
            node: ::sotu::StateOrSuperstate<'_, State, Superstate>,
            event: &(),
            context: &mut (),
        ) {
            ::sotu::blocking::DispatchHook::call_dispatch_hook(
                &(Self::count),
                self,
                node,
                event,
                context,
            )
        }
        fn after_dispatch<'ctx, 'data: 'ctx>(
            &mut self,
            node: ::sotu::StateOrSuperstate<'_, State, Superstate>,
            event: &(),
            context: &mut (),
        ) {
            ::sotu::blocking::DispatchHook::call_dispatch_hook(
                &(Self::count),
                self,
                node,
                event,
                context,
            )
        }
    }
    impl ::sotu::blocking::State<Lamp> for State {
        fn call_handler<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Lamp,
            _event: &(),
            _context: &mut (),
        ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
            match self {
                State::Dim {} => Lamp::dim(),
            }
        }
        fn superstate(&mut self) -> ::core::option::Option<Superstate> {
            match self {
                State::Dim {} => ::core::option::Option::Some(Superstate::Glowing {}),
            }
        }
    }
    impl ::sotu::blocking::Superstate<Lamp> for Superstate {
        fn call_handler<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Lamp,
            _event: &(),
            _context: &mut (),
        ) -> ::sotu::Outcome<State, ::core::convert::Infallible> {
            match self {
                Superstate::Glowing {} => Lamp::glowing(),
            }
        }
        fn superstate(&mut self) -> ::core::option::Option<Superstate> {
            match self {
                Superstate::Glowing {} => ::core::option::Option::None,
            }
        }
        fn same_state(&self, other: &Superstate) -> bool {
            match (self, other) {
                (Superstate::Glowing { .. }, Superstate::Glowing { .. }) => true,
            }
        }
    }
    impl ::sotu::private::Submachine for Lamp {
        type Event = ();
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = Superstate where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::dim()
        }
    }
    impl ::sotu::private::BlockingSubmachine for Lamp {}
    impl ::sotu::private::ExitlessSubmachine for Lamp {}
}
pub struct Desk {
    lamp: Option<lamp::Lamp>,
}
#[automatically_derived]
impl ::core::default::Default for Desk {
    #[inline]
    fn default() -> Desk {
        Desk {
            lamp: ::core::default::Default::default(),
        }
    }
}
impl Desk {
    fn lighting() -> Outcome<State> {
        Handled
    }
}
pub enum State {
    Lighting(<lamp::Lamp as ::sotu::private::Submachine>::State),
}
impl State {
    pub fn lighting(state: <lamp::Lamp as ::sotu::private::Submachine>::State) -> Self {
        Self::Lighting(state)
    }
}
pub enum Superstate<'sub> {
    Lighting(
        ::sotu::SubmachineNode<
            <lamp::Lamp as ::sotu::private::Submachine>::Superstate<'sub>,
        >,
    ),
}
impl Superstate<'_> {
    pub fn lighting() -> State {
        State::Lighting(<lamp::Lamp as ::sotu::private::Submachine>::initial())
    }
}
impl ::sotu::blocking::IntoStateMachine for Desk {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        Superstate::lighting()
    }
}
impl ::sotu::blocking::State<Desk> for State
where
    lamp::Lamp: ::sotu::private::BlockingSubmachine,
    lamp::Lamp: ::sotu::private::ExitlessSubmachine,
{
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Desk,
        _event: &(),
        _context: &mut (),
    ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
        match self {
            State::Lighting(_) => ::sotu::Outcome::Super,
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Desk,
        context: &mut (),
    ) {
        match self {
            State::Lighting(state) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    lamp::Lamp,
                >::get_mut(&mut shared_storage.lamp) {
                    <<lamp::Lamp as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                        lamp::Lamp,
                    >>::call_entry_action(state, submachine, context);
                }
            }
        }
    }
    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Desk,
        context: &mut (),
    ) {
        match self {
            State::Lighting(state) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    lamp::Lamp,
                >::get_mut(&mut shared_storage.lamp) {
                    <<lamp::Lamp as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                        lamp::Lamp,
                    >>::call_exit_action(state, submachine, context);
                }
            }
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate<'_>> {
        match self {
            State::Lighting(state) => {
                ::core::option::Option::Some(
                    Superstate::Lighting(
                        match <<lamp::Lamp as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            lamp::Lamp,
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
            (State::Lighting(first), State::Lighting(second)) => {
                <<lamp::Lamp as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                    lamp::Lamp,
                >>::same_state(first, second)
            }
        }
    }
    fn __is_addressed(&self, event: ()) -> (bool, ()) {
        match self {
            State::Lighting(node) => {
                match Desk::__project_into_lighting(
                    event,
                    |event| {
                        <<lamp::Lamp as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            lamp::Lamp,
                        >>::__is_addressed(node, event)
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
        shared_storage: &mut Desk,
        event: (),
        context: &mut (),
    ) -> ::sotu::private::Dispatched<Self, ::core::convert::Infallible, ()> {
        match self {
            State::Lighting(node) => {
                let mount_offset = <<lamp::Lamp as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::StateExt<
                    lamp::Lamp,
                >>::depth(node);
                let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    lamp::Lamp,
                >::get_mut(&mut shared_storage.lamp) else {
                    return ::sotu::private::Dispatched {
                        outcome: ::sotu::private::NodeOutcome::DeferToMount {
                            mount_offset,
                        },
                        event: event,
                    };
                };
                let visited = Desk::__project_into_lighting(
                    event,
                    |event| {
                        <lamp::Lamp as ::sotu::blocking::IntoStateMachine>::before_dispatch(
                            submachine,
                            ::sotu::StateOrSuperstate::State(&*node),
                            &event,
                            context,
                        );
                        let ::sotu::private::Dispatched { outcome, event } = <<lamp::Lamp as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            lamp::Lamp,
                        >>::__dispatch(node, submachine, event, context);
                        <lamp::Lamp as ::sotu::blocking::IntoStateMachine>::after_dispatch(
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
                                        State::Lighting(target),
                                    )
                                }
                                ::sotu::private::NodeOutcome::ExternalTransition {
                                    target,
                                    handler_offset,
                                } => {
                                    ::sotu::private::NodeOutcome::ExternalTransition {
                                        target: State::Lighting(target),
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
    fn __internal_transition_nesting(
        &mut self,
        target: &mut Self,
        exit_levels: usize,
    ) -> usize {
        match (self, target) {
            (State::Lighting(source), State::Lighting(target)) => {
                if exit_levels
                    <= <<lamp::Lamp as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::StateExt<
                        lamp::Lamp,
                    >>::depth(source)
                {
                    1
                        + <<lamp::Lamp as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            lamp::Lamp,
                        >>::__internal_transition_nesting(source, target, exit_levels)
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
        shared_storage: &mut Desk,
        context: &mut (),
    ) {
        match (self, target) {
            (State::Lighting(source), State::Lighting(target)) => {
                if exit_levels
                    > <<lamp::Lamp as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::StateExt<
                        lamp::Lamp,
                    >>::depth(source)
                {
                    return;
                }
                let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    lamp::Lamp,
                >::get_mut(&mut shared_storage.lamp) else {
                    return;
                };
                match nesting.checked_sub(1) {
                    ::core::option::Option::Some(0) => {
                        <lamp::Lamp as ::sotu::blocking::IntoStateMachine>::before_transition(
                            submachine,
                            source,
                            target,
                            context,
                        );
                    }
                    ::core::option::Option::Some(deeper) => {
                        <<lamp::Lamp as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            lamp::Lamp,
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
        }
    }
    fn __after_internal_transition<'ctx, 'data: 'ctx>(
        &mut self,
        target: &mut Self,
        exit_levels: usize,
        nesting: usize,
        shared_storage: &mut Desk,
        context: &mut (),
    ) {
        match (self, target) {
            (State::Lighting(source), State::Lighting(target)) => {
                if exit_levels
                    > <<lamp::Lamp as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::StateExt<
                        lamp::Lamp,
                    >>::depth(source)
                {
                    return;
                }
                let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    lamp::Lamp,
                >::get_mut(&mut shared_storage.lamp) else {
                    return;
                };
                match nesting.checked_sub(1) {
                    ::core::option::Option::Some(0) => {
                        <lamp::Lamp as ::sotu::blocking::IntoStateMachine>::after_transition(
                            submachine,
                            source,
                            target,
                            context,
                        );
                    }
                    ::core::option::Option::Some(deeper) => {
                        <<lamp::Lamp as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            lamp::Lamp,
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
        }
    }
}
impl<'sub> ::sotu::blocking::Superstate<Desk> for Superstate<'sub>
where
    lamp::Lamp: ::sotu::private::BlockingSubmachine,
    lamp::Lamp: ::sotu::private::ExitlessSubmachine,
{
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Desk,
        _event: &(),
        _context: &mut (),
    ) -> ::sotu::Outcome<State, ::core::convert::Infallible> {
        match self {
            Superstate::Lighting(::sotu::SubmachineNode::Inner(_)) => {
                ::sotu::Outcome::Super
            }
            Superstate::Lighting(::sotu::SubmachineNode::Mount) => Desk::lighting(),
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Desk,
        context: &mut (),
    ) {
        match self {
            Superstate::Lighting(::sotu::SubmachineNode::Mount) => {
                ::sotu::private::DefaultInstance::<
                    lamp::Lamp,
                >::fill_default(&mut shared_storage.lamp)
            }
            Superstate::Lighting(::sotu::SubmachineNode::Inner(state)) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    lamp::Lamp,
                >::get_mut(&mut shared_storage.lamp) {
                    <<lamp::Lamp as ::sotu::blocking::IntoStateMachine>::Superstate<
                        '_,
                    > as ::sotu::blocking::Superstate<
                        lamp::Lamp,
                    >>::call_entry_action(state, submachine, context);
                }
            }
        }
    }
    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Desk,
        context: &mut (),
    ) {
        match self {
            Superstate::Lighting(::sotu::SubmachineNode::Mount) => {
                ::sotu::private::MountField::<
                    lamp::Lamp,
                >::release(&mut shared_storage.lamp)
            }
            Superstate::Lighting(::sotu::SubmachineNode::Inner(state)) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    lamp::Lamp,
                >::get_mut(&mut shared_storage.lamp) {
                    <<lamp::Lamp as ::sotu::blocking::IntoStateMachine>::Superstate<
                        '_,
                    > as ::sotu::blocking::Superstate<
                        lamp::Lamp,
                    >>::call_exit_action(state, submachine, context);
                }
            }
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate<'_>> {
        match self {
            Superstate::Lighting(::sotu::SubmachineNode::Mount) => {
                ::core::option::Option::None
            }
            Superstate::Lighting(::sotu::SubmachineNode::Inner(superstate)) => {
                ::core::option::Option::Some(
                    Superstate::Lighting(
                        match <<lamp::Lamp as ::sotu::blocking::IntoStateMachine>::Superstate<
                            '_,
                        > as ::sotu::blocking::Superstate<
                            lamp::Lamp,
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
                Superstate::Lighting(::sotu::SubmachineNode::Mount),
                Superstate::Lighting(::sotu::SubmachineNode::Mount),
            ) => true,
            (
                Superstate::Lighting(::sotu::SubmachineNode::Inner(first)),
                Superstate::Lighting(::sotu::SubmachineNode::Inner(second)),
            ) => {
                <<lamp::Lamp as ::sotu::blocking::IntoStateMachine>::Superstate<
                    '_,
                > as ::sotu::blocking::Superstate<lamp::Lamp>>::same_state(first, second)
            }
            _ => false,
        }
    }
    fn __is_addressed(&self, event: ()) -> (bool, ()) {
        match self {
            Superstate::Lighting(::sotu::SubmachineNode::Inner(node)) => {
                match Desk::__project_into_lighting(
                    event,
                    |event| {
                        <<lamp::Lamp as ::sotu::blocking::IntoStateMachine>::Superstate<
                            '_,
                        > as ::sotu::blocking::Superstate<
                            lamp::Lamp,
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
        shared_storage: &mut Desk,
        event: (),
        context: &mut (),
    ) -> ::sotu::private::Dispatched<State, ::core::convert::Infallible, ()> {
        match self {
            Superstate::Lighting(::sotu::SubmachineNode::Inner(node)) => {
                let mount_offset = <<lamp::Lamp as ::sotu::blocking::IntoStateMachine>::Superstate<
                    '_,
                > as ::sotu::blocking::SuperstateExt<lamp::Lamp>>::depth(node);
                let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    lamp::Lamp,
                >::get_mut(&mut shared_storage.lamp) else {
                    return ::sotu::private::Dispatched {
                        outcome: ::sotu::private::NodeOutcome::DeferToMount {
                            mount_offset,
                        },
                        event: event,
                    };
                };
                let visited = Desk::__project_into_lighting(
                    event,
                    |event| {
                        <lamp::Lamp as ::sotu::blocking::IntoStateMachine>::before_dispatch(
                            submachine,
                            ::sotu::StateOrSuperstate::Superstate(&*node),
                            &event,
                            context,
                        );
                        let ::sotu::private::Dispatched { outcome, event } = <<lamp::Lamp as ::sotu::blocking::IntoStateMachine>::Superstate<
                            '_,
                        > as ::sotu::blocking::Superstate<
                            lamp::Lamp,
                        >>::__dispatch(node, submachine, event, context);
                        <lamp::Lamp as ::sotu::blocking::IntoStateMachine>::after_dispatch(
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
                                        State::Lighting(target),
                                    )
                                }
                                ::sotu::private::NodeOutcome::ExternalTransition {
                                    target,
                                    handler_offset,
                                } => {
                                    ::sotu::private::NodeOutcome::ExternalTransition {
                                        target: State::Lighting(target),
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
                    Desk,
                >::call_handler(self, shared_storage, &event, context);
                ::sotu::private::Dispatched {
                    outcome: ::sotu::private::NodeOutcome::from(outcome),
                    event: event,
                }
            }
        }
    }
}
impl ::sotu::private::Submachine for Desk {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        Superstate::lighting()
    }
}
impl ::sotu::private::BlockingSubmachine for Desk {}
impl ::sotu::private::ExitlessSubmachine for Desk {}
impl Desk {
    #[doc(hidden)]
    #[allow(unreachable_code)]
    fn __project_into_lighting<Visited>(
        event: (),
        visit: impl ::core::ops::FnOnce(
            <lamp::Lamp as ::sotu::private::Submachine>::Event,
        ) -> (Visited, <lamp::Lamp as ::sotu::private::Submachine>::Event),
    ) -> ::core::result::Result<(Visited, ()), ()> {
        if !<<lamp::Lamp as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
            (),
        >>::accepts(&event) {
            return ::core::result::Result::Err(event);
        }
        match <<lamp::Lamp as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
            (),
        >>::project(event) {
            ::core::result::Result::Ok(child_event) => {
                let (visited, child_event) = visit(child_event);
                ::core::result::Result::Ok((
                    visited,
                    <<lamp::Lamp as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
                        (),
                    >>::embed(child_event),
                ))
            }
            ::core::result::Result::Err(event) => ::core::result::Result::Err(event),
        }
    }
}
fn main() {
    let mut machine = Desk::default().state_machine();
    machine.step(&mut ());
}
