use sotu::prelude::*;
mod bell {
    use sotu::prelude::*;
    pub struct Bell;
    #[automatically_derived]
    impl ::core::default::Default for Bell {
        #[inline]
        fn default() -> Bell {
            Bell {}
        }
    }
    pub struct Ring;
    pub enum Event {
        Ring(Ring),
    }
    impl ::core::convert::From<Ring> for Event {
        fn from(value: Ring) -> Self {
            Self::Ring(value)
        }
    }
    impl ::sotu::event::Variant<Ring> for Event {
        fn from_variant(value: Ring) -> Self {
            Self::Ring(value)
        }
        fn into_variant(self) -> ::core::result::Result<Ring, Self> {
            match self {
                Self::Ring(value) => ::core::result::Result::Ok(value),
            }
        }
        fn as_variant_ref(&self) -> ::core::option::Option<&Ring> {
            match self {
                Self::Ring(value) => ::core::option::Option::Some(value),
            }
        }
    }
    impl<SotuParentEvent> ::sotu::event::SubEvent<SotuParentEvent> for Event
    where
        SotuParentEvent: ::sotu::event::Variant<Ring>,
    {
        fn accepts(parent: &SotuParentEvent) -> bool {
            <SotuParentEvent as ::sotu::event::Variant<Ring>>::as_variant_ref(parent)
                .is_some()
        }
        fn project(
            parent: SotuParentEvent,
        ) -> ::core::result::Result<Self, SotuParentEvent> {
            let parent = match <SotuParentEvent as ::sotu::event::Variant<
                Ring,
            >>::into_variant(parent) {
                ::core::result::Result::Ok(value) => {
                    return ::core::result::Result::Ok(Self::Ring(value));
                }
                ::core::result::Result::Err(parent) => parent,
            };
            ::core::result::Result::Err(parent)
        }
        fn embed(self) -> SotuParentEvent {
            match self {
                Self::Ring(value) => {
                    <SotuParentEvent as ::sotu::event::Variant<
                        Ring,
                    >>::from_variant(value)
                }
            }
        }
    }
    impl Bell {
        fn silent(event: &Event) -> Outcome<State> {
            match event {
                Event::Ring(_) => Handled,
            }
        }
    }
    pub enum State {
        Silent {},
    }
    impl State {
        pub fn silent() -> Self {
            Self::Silent {}
        }
    }
    impl ::sotu::blocking::IntoStateMachine for Bell {
        type Event = Event;
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = () where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::silent()
        }
    }
    impl ::sotu::blocking::State<Bell> for State {
        fn call_handler<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Bell,
            event: &Event,
            _context: &mut (),
        ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
            match self {
                State::Silent {} => Bell::silent(event),
            }
        }
    }
    impl ::sotu::private::Submachine for Bell {
        type Event = Event;
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = () where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::silent()
        }
    }
    impl ::sotu::private::BlockingSubmachine for Bell {}
    impl ::sotu::private::ExitlessSubmachine for Bell {}
}
pub struct Enter;
pub enum Event {
    Enter(Enter),
}
impl ::core::convert::From<Enter> for Event {
    fn from(value: Enter) -> Self {
        Self::Enter(value)
    }
}
impl ::sotu::event::Variant<Enter> for Event {
    fn from_variant(value: Enter) -> Self {
        Self::Enter(value)
    }
    fn into_variant(self) -> ::core::result::Result<Enter, Self> {
        match self {
            Self::Enter(value) => ::core::result::Result::Ok(value),
        }
    }
    fn as_variant_ref(&self) -> ::core::option::Option<&Enter> {
        match self {
            Self::Enter(value) => ::core::option::Option::Some(value),
        }
    }
}
impl<SotuParentEvent> ::sotu::event::SubEvent<SotuParentEvent> for Event
where
    SotuParentEvent: ::sotu::event::Variant<Enter>,
{
    fn accepts(parent: &SotuParentEvent) -> bool {
        <SotuParentEvent as ::sotu::event::Variant<Enter>>::as_variant_ref(parent)
            .is_some()
    }
    fn project(
        parent: SotuParentEvent,
    ) -> ::core::result::Result<Self, SotuParentEvent> {
        let parent = match <SotuParentEvent as ::sotu::event::Variant<
            Enter,
        >>::into_variant(parent) {
            ::core::result::Result::Ok(value) => {
                return ::core::result::Result::Ok(Self::Enter(value));
            }
            ::core::result::Result::Err(parent) => parent,
        };
        ::core::result::Result::Err(parent)
    }
    fn embed(self) -> SotuParentEvent {
        match self {
            Self::Enter(value) => {
                <SotuParentEvent as ::sotu::event::Variant<Enter>>::from_variant(value)
            }
        }
    }
}
pub struct House {
    bell: Option<bell::Bell>,
}
#[automatically_derived]
impl ::core::default::Default for House {
    #[inline]
    fn default() -> House {
        House {
            bell: ::core::default::Default::default(),
        }
    }
}
impl House {
    fn idle(event: &Event) -> Outcome<State> {
        match event {
            Event::Enter(_) => Transition(Superstate::ringing()),
        }
    }
    fn ringing() -> Outcome<State> {
        Handled
    }
}
pub enum State {
    Idle {},
    Ringing(<bell::Bell as ::sotu::private::Submachine>::State),
}
impl State {
    pub fn idle() -> Self {
        Self::Idle {}
    }
    pub fn ringing(state: <bell::Bell as ::sotu::private::Submachine>::State) -> Self {
        Self::Ringing(state)
    }
}
pub enum Superstate<'sub> {
    Ringing(
        ::sotu::SubmachineNode<
            <bell::Bell as ::sotu::private::Submachine>::Superstate<'sub>,
        >,
    ),
}
impl Superstate<'_> {
    pub fn ringing() -> State {
        State::Ringing(<bell::Bell as ::sotu::private::Submachine>::initial())
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
    bell::Bell: ::sotu::private::BlockingSubmachine,
    bell::Bell: ::sotu::private::ExitlessSubmachine,
{
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut House,
        event: &Event,
        _context: &mut (),
    ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
        match self {
            State::Idle {} => House::idle(event),
            State::Ringing(_) => ::sotu::Outcome::Super,
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut House,
        context: &mut (),
    ) {
        match self {
            State::Idle { .. } => {}
            State::Ringing(state) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    bell::Bell,
                >::get_mut(&mut shared_storage.bell) {
                    <<bell::Bell as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                        bell::Bell,
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
            State::Ringing(state) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    bell::Bell,
                >::get_mut(&mut shared_storage.bell) {
                    <<bell::Bell as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                        bell::Bell,
                    >>::call_exit_action(state, submachine, context);
                }
            }
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate<'_>> {
        match self {
            State::Idle {} => ::core::option::Option::None,
            State::Ringing(state) => {
                ::core::option::Option::Some(
                    Superstate::Ringing(
                        match <<bell::Bell as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            bell::Bell,
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
            (State::Ringing(first), State::Ringing(second)) => {
                <<bell::Bell as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                    bell::Bell,
                >>::same_state(first, second)
            }
            _ => false,
        }
    }
    fn __is_addressed(&self, event: Event) -> (bool, Event) {
        match self {
            State::Ringing(node) => {
                match House::__project_into_ringing(
                    event,
                    |event| {
                        <<bell::Bell as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            bell::Bell,
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
            State::Ringing(node) => {
                let mount_offset = <<bell::Bell as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::StateExt<
                    bell::Bell,
                >>::depth(node);
                let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    bell::Bell,
                >::get_mut(&mut shared_storage.bell) else {
                    return ::sotu::private::Dispatched {
                        outcome: ::sotu::private::NodeOutcome::DeferToMount {
                            mount_offset,
                        },
                        event: event,
                    };
                };
                let visited = House::__project_into_ringing(
                    event,
                    |event| {
                        <bell::Bell as ::sotu::blocking::IntoStateMachine>::before_dispatch(
                            submachine,
                            ::sotu::StateOrSuperstate::State(&*node),
                            &event,
                            context,
                        );
                        let ::sotu::private::Dispatched { outcome, event } = <<bell::Bell as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            bell::Bell,
                        >>::__dispatch(node, submachine, event, context);
                        <bell::Bell as ::sotu::blocking::IntoStateMachine>::after_dispatch(
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
                                        State::Ringing(target),
                                    )
                                }
                                ::sotu::private::NodeOutcome::ExternalTransition {
                                    target,
                                    handler_offset,
                                } => {
                                    ::sotu::private::NodeOutcome::ExternalTransition {
                                        target: State::Ringing(target),
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
            (State::Ringing(source), State::Ringing(target)) => {
                if exit_levels
                    <= <<bell::Bell as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::StateExt<
                        bell::Bell,
                    >>::depth(source)
                {
                    1
                        + <<bell::Bell as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            bell::Bell,
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
            (State::Ringing(source), State::Ringing(target)) => {
                if exit_levels
                    > <<bell::Bell as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::StateExt<
                        bell::Bell,
                    >>::depth(source)
                {
                    return;
                }
                let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    bell::Bell,
                >::get_mut(&mut shared_storage.bell) else {
                    return;
                };
                match nesting.checked_sub(1) {
                    ::core::option::Option::Some(0) => {
                        <bell::Bell as ::sotu::blocking::IntoStateMachine>::before_transition(
                            submachine,
                            source,
                            target,
                            context,
                        );
                    }
                    ::core::option::Option::Some(deeper) => {
                        <<bell::Bell as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            bell::Bell,
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
            (State::Ringing(source), State::Ringing(target)) => {
                if exit_levels
                    > <<bell::Bell as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::StateExt<
                        bell::Bell,
                    >>::depth(source)
                {
                    return;
                }
                let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    bell::Bell,
                >::get_mut(&mut shared_storage.bell) else {
                    return;
                };
                match nesting.checked_sub(1) {
                    ::core::option::Option::Some(0) => {
                        <bell::Bell as ::sotu::blocking::IntoStateMachine>::after_transition(
                            submachine,
                            source,
                            target,
                            context,
                        );
                    }
                    ::core::option::Option::Some(deeper) => {
                        <<bell::Bell as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            bell::Bell,
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
    bell::Bell: ::sotu::private::BlockingSubmachine,
    bell::Bell: ::sotu::private::ExitlessSubmachine,
{
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut House,
        _event: &Event,
        _context: &mut (),
    ) -> ::sotu::Outcome<State, ::core::convert::Infallible> {
        match self {
            Superstate::Ringing(::sotu::SubmachineNode::Inner(_)) => {
                ::sotu::Outcome::Super
            }
            Superstate::Ringing(::sotu::SubmachineNode::Mount) => House::ringing(),
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut House,
        context: &mut (),
    ) {
        match self {
            Superstate::Ringing(::sotu::SubmachineNode::Mount) => {
                ::sotu::private::DefaultInstance::<
                    bell::Bell,
                >::fill_default(&mut shared_storage.bell)
            }
            Superstate::Ringing(::sotu::SubmachineNode::Inner(state)) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    bell::Bell,
                >::get_mut(&mut shared_storage.bell) {
                    <<bell::Bell as ::sotu::blocking::IntoStateMachine>::Superstate<
                        '_,
                    > as ::sotu::blocking::Superstate<
                        bell::Bell,
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
            Superstate::Ringing(::sotu::SubmachineNode::Mount) => {
                ::sotu::private::MountField::<
                    bell::Bell,
                >::release(&mut shared_storage.bell)
            }
            Superstate::Ringing(::sotu::SubmachineNode::Inner(state)) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    bell::Bell,
                >::get_mut(&mut shared_storage.bell) {
                    <<bell::Bell as ::sotu::blocking::IntoStateMachine>::Superstate<
                        '_,
                    > as ::sotu::blocking::Superstate<
                        bell::Bell,
                    >>::call_exit_action(state, submachine, context);
                }
            }
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate<'_>> {
        match self {
            Superstate::Ringing(::sotu::SubmachineNode::Mount) => {
                ::core::option::Option::None
            }
            Superstate::Ringing(::sotu::SubmachineNode::Inner(superstate)) => {
                ::core::option::Option::Some(
                    Superstate::Ringing(
                        match <<bell::Bell as ::sotu::blocking::IntoStateMachine>::Superstate<
                            '_,
                        > as ::sotu::blocking::Superstate<
                            bell::Bell,
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
                Superstate::Ringing(::sotu::SubmachineNode::Mount),
                Superstate::Ringing(::sotu::SubmachineNode::Mount),
            ) => true,
            (
                Superstate::Ringing(::sotu::SubmachineNode::Inner(first)),
                Superstate::Ringing(::sotu::SubmachineNode::Inner(second)),
            ) => {
                <<bell::Bell as ::sotu::blocking::IntoStateMachine>::Superstate<
                    '_,
                > as ::sotu::blocking::Superstate<bell::Bell>>::same_state(first, second)
            }
            _ => false,
        }
    }
    fn __is_addressed(&self, event: Event) -> (bool, Event) {
        match self {
            Superstate::Ringing(::sotu::SubmachineNode::Inner(node)) => {
                match House::__project_into_ringing(
                    event,
                    |event| {
                        <<bell::Bell as ::sotu::blocking::IntoStateMachine>::Superstate<
                            '_,
                        > as ::sotu::blocking::Superstate<
                            bell::Bell,
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
            Superstate::Ringing(::sotu::SubmachineNode::Inner(node)) => {
                let mount_offset = <<bell::Bell as ::sotu::blocking::IntoStateMachine>::Superstate<
                    '_,
                > as ::sotu::blocking::SuperstateExt<bell::Bell>>::depth(node);
                let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    bell::Bell,
                >::get_mut(&mut shared_storage.bell) else {
                    return ::sotu::private::Dispatched {
                        outcome: ::sotu::private::NodeOutcome::DeferToMount {
                            mount_offset,
                        },
                        event: event,
                    };
                };
                let visited = House::__project_into_ringing(
                    event,
                    |event| {
                        <bell::Bell as ::sotu::blocking::IntoStateMachine>::before_dispatch(
                            submachine,
                            ::sotu::StateOrSuperstate::Superstate(&*node),
                            &event,
                            context,
                        );
                        let ::sotu::private::Dispatched { outcome, event } = <<bell::Bell as ::sotu::blocking::IntoStateMachine>::Superstate<
                            '_,
                        > as ::sotu::blocking::Superstate<
                            bell::Bell,
                        >>::__dispatch(node, submachine, event, context);
                        <bell::Bell as ::sotu::blocking::IntoStateMachine>::after_dispatch(
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
                                        State::Ringing(target),
                                    )
                                }
                                ::sotu::private::NodeOutcome::ExternalTransition {
                                    target,
                                    handler_offset,
                                } => {
                                    ::sotu::private::NodeOutcome::ExternalTransition {
                                        target: State::Ringing(target),
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
    fn __project_into_ringing<Visited>(
        event: Event,
        visit: impl ::core::ops::FnOnce(
            <bell::Bell as ::sotu::private::Submachine>::Event,
        ) -> (Visited, <bell::Bell as ::sotu::private::Submachine>::Event),
    ) -> ::core::result::Result<(Visited, Event), Event> {
        if !<<bell::Bell as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
            Event,
        >>::accepts(&event) {
            return ::core::result::Result::Err(event);
        }
        match <<bell::Bell as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
            Event,
        >>::project(event) {
            ::core::result::Result::Ok(child_event) => {
                let (visited, child_event) = visit(child_event);
                ::core::result::Result::Ok((
                    visited,
                    <<bell::Bell as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
                        Event,
                    >>::embed(child_event),
                ))
            }
            ::core::result::Result::Err(event) => ::core::result::Result::Err(event),
        }
    }
}
fn main() {}
