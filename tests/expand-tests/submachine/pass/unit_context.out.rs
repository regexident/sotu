use sotu::prelude::*;
mod lamp {
    use sotu::prelude::*;
    pub struct Lamp {
        pub toggles: u32,
    }
    #[automatically_derived]
    impl ::core::default::Default for Lamp {
        #[inline]
        fn default() -> Lamp {
            Lamp {
                toggles: ::core::default::Default::default(),
            }
        }
    }
    impl Lamp {
        fn off(&mut self) -> Outcome<State> {
            self.toggles += 1;
            Handled
        }
        fn reset(&mut self) {
            self.toggles = 0;
        }
    }
    pub enum State {
        Off {},
    }
    impl State {
        pub fn off() -> Self {
            Self::Off {}
        }
    }
    impl ::sotu::blocking::IntoStateMachine for Lamp {
        type Event = ();
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = () where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::off()
        }
    }
    impl ::sotu::blocking::State<Lamp> for State {
        fn call_handler<'ctx, 'data: 'ctx>(
            &mut self,
            shared_storage: &mut Lamp,
            _event: &(),
            _context: &mut (),
        ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
            match self {
                State::Off {} => Lamp::off(shared_storage),
            }
        }
        fn call_entry_action<'ctx, 'data: 'ctx>(
            &mut self,
            shared_storage: &mut Lamp,
            _context: &mut (),
        ) {
            match self {
                State::Off {} => Lamp::reset(shared_storage),
            }
        }
    }
    impl ::sotu::private::Submachine for Lamp {
        type Event = ();
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = () where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::off()
        }
    }
    impl ::sotu::private::BlockingSubmachine for Lamp {}
    impl ::sotu::private::ExitlessSubmachine for Lamp {}
}
pub struct Room {
    lamp: Option<lamp::Lamp>,
}
#[automatically_derived]
impl ::core::default::Default for Room {
    #[inline]
    fn default() -> Room {
        Room {
            lamp: ::core::default::Default::default(),
        }
    }
}
impl Room {
    fn lit(context: &mut u32) -> Outcome<State> {
        *context += 1;
        Handled
    }
}
pub enum State {
    Lit(<lamp::Lamp as ::sotu::private::Submachine>::State),
}
impl State {
    pub fn lit(state: <lamp::Lamp as ::sotu::private::Submachine>::State) -> Self {
        Self::Lit(state)
    }
}
pub enum Superstate<'sub> {
    Lit(
        ::sotu::SubmachineNode<
            <lamp::Lamp as ::sotu::private::Submachine>::Superstate<'sub>,
        >,
    ),
}
impl Superstate<'_> {
    pub fn lit() -> State {
        State::Lit(<lamp::Lamp as ::sotu::private::Submachine>::initial())
    }
}
impl ::sotu::blocking::IntoStateMachine for Room {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = u32;
    type State = State;
    type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        Superstate::lit()
    }
}
impl ::sotu::blocking::State<Room> for State
where
    lamp::Lamp: ::sotu::private::BlockingSubmachine,
    lamp::Lamp: ::sotu::private::ExitlessSubmachine,
{
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Room,
        _event: &(),
        _context: &mut u32,
    ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
        match self {
            State::Lit(_) => ::sotu::Outcome::Super,
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Room,
        context: &mut u32,
    ) {
        match self {
            State::Lit(state) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    lamp::Lamp,
                >::get_mut(&mut shared_storage.lamp) {
                    {
                        let mut child_context = (sotu::event::unit_context)(context);
                        <<lamp::Lamp as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            lamp::Lamp,
                        >>::call_entry_action(state, submachine, &mut child_context)
                    };
                }
            }
        }
    }
    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Room,
        context: &mut u32,
    ) {
        match self {
            State::Lit(state) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    lamp::Lamp,
                >::get_mut(&mut shared_storage.lamp) {
                    {
                        let mut child_context = (sotu::event::unit_context)(context);
                        <<lamp::Lamp as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            lamp::Lamp,
                        >>::call_exit_action(state, submachine, &mut child_context)
                    };
                }
            }
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate<'_>> {
        match self {
            State::Lit(state) => {
                ::core::option::Option::Some(
                    Superstate::Lit(
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
            (State::Lit(first), State::Lit(second)) => {
                <<lamp::Lamp as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                    lamp::Lamp,
                >>::same_state(first, second)
            }
        }
    }
    fn __is_addressed(&self, event: ()) -> (bool, ()) {
        match self {
            State::Lit(node) => {
                match Room::__project_into_lit(
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
        shared_storage: &mut Room,
        event: (),
        context: &mut u32,
    ) -> ::sotu::private::Dispatched<Self, ::core::convert::Infallible, ()> {
        match self {
            State::Lit(node) => {
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
                let visited = Room::__project_into_lit(
                    event,
                    |event| {
                        {
                            let mut child_context = (sotu::event::unit_context)(context);
                            <lamp::Lamp as ::sotu::blocking::IntoStateMachine>::before_dispatch(
                                submachine,
                                ::sotu::StateOrSuperstate::State(&*node),
                                &event,
                                &mut child_context,
                            )
                        };
                        let ::sotu::private::Dispatched { outcome, event } = {
                            let mut child_context = (sotu::event::unit_context)(context);
                            <<lamp::Lamp as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                                lamp::Lamp,
                            >>::__dispatch(node, submachine, event, &mut child_context)
                        };
                        {
                            let mut child_context = (sotu::event::unit_context)(context);
                            <lamp::Lamp as ::sotu::blocking::IntoStateMachine>::after_dispatch(
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
                                    ::sotu::private::NodeOutcome::Transition(State::Lit(target))
                                }
                                ::sotu::private::NodeOutcome::ExternalTransition {
                                    target,
                                    handler_offset,
                                } => {
                                    ::sotu::private::NodeOutcome::ExternalTransition {
                                        target: State::Lit(target),
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
            (State::Lit(source), State::Lit(target)) => {
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
        shared_storage: &mut Room,
        context: &mut u32,
    ) {
        match (self, target) {
            (State::Lit(source), State::Lit(target)) => {
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
                        {
                            let mut child_context = (sotu::event::unit_context)(context);
                            <lamp::Lamp as ::sotu::blocking::IntoStateMachine>::before_transition(
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
                            <<lamp::Lamp as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                                lamp::Lamp,
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
        }
    }
    fn __after_internal_transition<'ctx, 'data: 'ctx>(
        &mut self,
        target: &mut Self,
        exit_levels: usize,
        nesting: usize,
        shared_storage: &mut Room,
        context: &mut u32,
    ) {
        match (self, target) {
            (State::Lit(source), State::Lit(target)) => {
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
                        {
                            let mut child_context = (sotu::event::unit_context)(context);
                            <lamp::Lamp as ::sotu::blocking::IntoStateMachine>::after_transition(
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
                            <<lamp::Lamp as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                                lamp::Lamp,
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
        }
    }
}
impl<'sub> ::sotu::blocking::Superstate<Room> for Superstate<'sub>
where
    lamp::Lamp: ::sotu::private::BlockingSubmachine,
    lamp::Lamp: ::sotu::private::ExitlessSubmachine,
{
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Room,
        _event: &(),
        context: &mut u32,
    ) -> ::sotu::Outcome<State, ::core::convert::Infallible> {
        match self {
            Superstate::Lit(::sotu::SubmachineNode::Inner(_)) => ::sotu::Outcome::Super,
            Superstate::Lit(::sotu::SubmachineNode::Mount) => Room::lit(context),
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Room,
        context: &mut u32,
    ) {
        match self {
            Superstate::Lit(::sotu::SubmachineNode::Mount) => {
                ::sotu::private::DefaultInstance::<
                    lamp::Lamp,
                >::fill_default(&mut shared_storage.lamp)
            }
            Superstate::Lit(::sotu::SubmachineNode::Inner(state)) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    lamp::Lamp,
                >::get_mut(&mut shared_storage.lamp) {
                    {
                        let mut child_context = (sotu::event::unit_context)(context);
                        <<lamp::Lamp as ::sotu::blocking::IntoStateMachine>::Superstate<
                            '_,
                        > as ::sotu::blocking::Superstate<
                            lamp::Lamp,
                        >>::call_entry_action(state, submachine, &mut child_context)
                    };
                }
            }
        }
    }
    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Room,
        context: &mut u32,
    ) {
        match self {
            Superstate::Lit(::sotu::SubmachineNode::Mount) => {
                ::sotu::private::MountField::<
                    lamp::Lamp,
                >::release(&mut shared_storage.lamp)
            }
            Superstate::Lit(::sotu::SubmachineNode::Inner(state)) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    lamp::Lamp,
                >::get_mut(&mut shared_storage.lamp) {
                    {
                        let mut child_context = (sotu::event::unit_context)(context);
                        <<lamp::Lamp as ::sotu::blocking::IntoStateMachine>::Superstate<
                            '_,
                        > as ::sotu::blocking::Superstate<
                            lamp::Lamp,
                        >>::call_exit_action(state, submachine, &mut child_context)
                    };
                }
            }
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate<'_>> {
        match self {
            Superstate::Lit(::sotu::SubmachineNode::Mount) => {
                ::core::option::Option::None
            }
            Superstate::Lit(::sotu::SubmachineNode::Inner(superstate)) => {
                ::core::option::Option::Some(
                    Superstate::Lit(
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
                Superstate::Lit(::sotu::SubmachineNode::Mount),
                Superstate::Lit(::sotu::SubmachineNode::Mount),
            ) => true,
            (
                Superstate::Lit(::sotu::SubmachineNode::Inner(first)),
                Superstate::Lit(::sotu::SubmachineNode::Inner(second)),
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
            Superstate::Lit(::sotu::SubmachineNode::Inner(node)) => {
                match Room::__project_into_lit(
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
        shared_storage: &mut Room,
        event: (),
        context: &mut u32,
    ) -> ::sotu::private::Dispatched<State, ::core::convert::Infallible, ()> {
        match self {
            Superstate::Lit(::sotu::SubmachineNode::Inner(node)) => {
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
                let visited = Room::__project_into_lit(
                    event,
                    |event| {
                        {
                            let mut child_context = (sotu::event::unit_context)(context);
                            <lamp::Lamp as ::sotu::blocking::IntoStateMachine>::before_dispatch(
                                submachine,
                                ::sotu::StateOrSuperstate::Superstate(&*node),
                                &event,
                                &mut child_context,
                            )
                        };
                        let ::sotu::private::Dispatched { outcome, event } = {
                            let mut child_context = (sotu::event::unit_context)(context);
                            <<lamp::Lamp as ::sotu::blocking::IntoStateMachine>::Superstate<
                                '_,
                            > as ::sotu::blocking::Superstate<
                                lamp::Lamp,
                            >>::__dispatch(node, submachine, event, &mut child_context)
                        };
                        {
                            let mut child_context = (sotu::event::unit_context)(context);
                            <lamp::Lamp as ::sotu::blocking::IntoStateMachine>::after_dispatch(
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
                                    ::sotu::private::NodeOutcome::Transition(State::Lit(target))
                                }
                                ::sotu::private::NodeOutcome::ExternalTransition {
                                    target,
                                    handler_offset,
                                } => {
                                    ::sotu::private::NodeOutcome::ExternalTransition {
                                        target: State::Lit(target),
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
    type Context<'ctx, 'data: 'ctx> = u32;
    type State = State;
    type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        Superstate::lit()
    }
}
impl ::sotu::private::BlockingSubmachine for Room {}
impl ::sotu::private::ExitlessSubmachine for Room {}
impl Room {
    #[doc(hidden)]
    #[allow(unreachable_code)]
    fn __project_into_lit<Visited>(
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
    let mut context = 0;
    let mut machine = Room::default().state_machine();
    machine.step(&mut context);
}
