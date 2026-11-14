use sotu::prelude::*;
mod inner {
    use sotu::prelude::*;
    pub struct Inner;
    #[automatically_derived]
    impl ::core::default::Default for Inner {
        #[inline]
        fn default() -> Inner {
            Inner {}
        }
    }
    impl Inner {
        fn leaf() -> Outcome<State> {
            Handled
        }
    }
    pub enum State {
        Leaf {},
    }
    impl State {
        pub fn leaf() -> Self {
            Self::Leaf {}
        }
    }
    impl ::sotu::blocking::IntoStateMachine for Inner {
        type Event = ();
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = () where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::leaf()
        }
    }
    impl ::sotu::blocking::State<Inner> for State {
        fn call_handler<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Inner,
            _event: &(),
            _context: &mut (),
        ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
            match self {
                State::Leaf {} => Inner::leaf(),
            }
        }
    }
    impl ::sotu::private::Submachine for Inner {
        type Event = ();
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = () where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::leaf()
        }
    }
    impl ::sotu::private::BlockingSubmachine for Inner {}
    impl ::sotu::private::ExitlessSubmachine for Inner {}
}
mod outer {
    use sotu::prelude::*;
    use super::inner;
    pub struct Outer {
        inner: Option<inner::Inner>,
    }
    #[automatically_derived]
    impl ::core::default::Default for Outer {
        #[inline]
        fn default() -> Outer {
            Outer {
                inner: ::core::default::Default::default(),
            }
        }
    }
    impl Outer {
        fn nested() -> Outcome<State> {
            Handled
        }
    }
    pub enum State {
        Nested(<inner::Inner as ::sotu::private::Submachine>::State),
    }
    impl State {
        pub fn nested(
            state: <inner::Inner as ::sotu::private::Submachine>::State,
        ) -> Self {
            Self::Nested(state)
        }
    }
    pub enum Superstate<'sub> {
        Nested(
            ::sotu::SubmachineNode<
                <inner::Inner as ::sotu::private::Submachine>::Superstate<'sub>,
            >,
        ),
    }
    impl Superstate<'_> {
        pub fn nested() -> State {
            State::Nested(<inner::Inner as ::sotu::private::Submachine>::initial())
        }
    }
    impl ::sotu::blocking::IntoStateMachine for Outer {
        type Event = ();
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            Superstate::nested()
        }
    }
    impl ::sotu::blocking::State<Outer> for State
    where
        inner::Inner: ::sotu::private::BlockingSubmachine,
        inner::Inner: ::sotu::private::ExitlessSubmachine,
    {
        fn call_handler<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Outer,
            _event: &(),
            _context: &mut (),
        ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
            match self {
                State::Nested(_) => ::sotu::Outcome::Super,
            }
        }
        fn call_entry_action<'ctx, 'data: 'ctx>(
            &mut self,
            shared_storage: &mut Outer,
            context: &mut (),
        ) {
            match self {
                State::Nested(state) => {
                    if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                        inner::Inner,
                    >::get_mut(&mut shared_storage.inner) {
                        <<inner::Inner as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            inner::Inner,
                        >>::call_entry_action(state, submachine, context);
                    }
                }
            }
        }
        fn call_exit_action<'ctx, 'data: 'ctx>(
            &mut self,
            shared_storage: &mut Outer,
            context: &mut (),
        ) {
            match self {
                State::Nested(state) => {
                    if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                        inner::Inner,
                    >::get_mut(&mut shared_storage.inner) {
                        <<inner::Inner as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            inner::Inner,
                        >>::call_exit_action(state, submachine, context);
                    }
                }
            }
        }
        fn superstate(&mut self) -> ::core::option::Option<Superstate<'_>> {
            match self {
                State::Nested(state) => {
                    ::core::option::Option::Some(
                        Superstate::Nested(
                            match <<inner::Inner as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                                inner::Inner,
                            >>::superstate(state) {
                                ::core::option::Option::Some(superstate) => {
                                    ::sotu::SubmachineNode::Inner(superstate)
                                }
                                ::core::option::Option::None => {
                                    ::sotu::SubmachineNode::Mount
                                }
                            },
                        ),
                    )
                }
            }
        }
        fn same_state(&self, other: &Self) -> bool {
            match (self, other) {
                (State::Nested(first), State::Nested(second)) => {
                    <<inner::Inner as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                        inner::Inner,
                    >>::same_state(first, second)
                }
            }
        }
        fn __is_addressed(&self, event: ()) -> (bool, ()) {
            match self {
                State::Nested(node) => {
                    match Outer::__project_into_nested(
                        event,
                        |event| {
                            <<inner::Inner as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                                inner::Inner,
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
            shared_storage: &mut Outer,
            event: (),
            context: &mut (),
        ) -> ::sotu::private::Dispatched<Self, ::core::convert::Infallible, ()> {
            match self {
                State::Nested(node) => {
                    let mount_offset = <<inner::Inner as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::StateExt<
                        inner::Inner,
                    >>::depth(node);
                    let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                        inner::Inner,
                    >::get_mut(&mut shared_storage.inner) else {
                        return ::sotu::private::Dispatched {
                            outcome: ::sotu::private::NodeOutcome::DeferToMount {
                                mount_offset,
                            },
                            event: event,
                        };
                    };
                    let visited = Outer::__project_into_nested(
                        event,
                        |event| {
                            let ::sotu::private::Dispatched { outcome, event } = <<inner::Inner as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                                inner::Inner,
                            >>::__dispatch(node, submachine, event, context);
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
                                            State::Nested(target),
                                        )
                                    }
                                    ::sotu::private::NodeOutcome::ExternalTransition {
                                        target,
                                        handler_offset,
                                    } => {
                                        ::sotu::private::NodeOutcome::ExternalTransition {
                                            target: State::Nested(target),
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
    impl<'sub> ::sotu::blocking::Superstate<Outer> for Superstate<'sub>
    where
        inner::Inner: ::sotu::private::BlockingSubmachine,
        inner::Inner: ::sotu::private::ExitlessSubmachine,
    {
        fn call_handler<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Outer,
            _event: &(),
            _context: &mut (),
        ) -> ::sotu::Outcome<State, ::core::convert::Infallible> {
            match self {
                Superstate::Nested(::sotu::SubmachineNode::Inner(_)) => {
                    ::sotu::Outcome::Super
                }
                Superstate::Nested(::sotu::SubmachineNode::Mount) => Outer::nested(),
            }
        }
        fn call_entry_action<'ctx, 'data: 'ctx>(
            &mut self,
            shared_storage: &mut Outer,
            context: &mut (),
        ) {
            match self {
                Superstate::Nested(::sotu::SubmachineNode::Mount) => {
                    ::sotu::private::DefaultInstance::<
                        inner::Inner,
                    >::fill_default(&mut shared_storage.inner)
                }
                Superstate::Nested(::sotu::SubmachineNode::Inner(state)) => {
                    if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                        inner::Inner,
                    >::get_mut(&mut shared_storage.inner) {
                        <<inner::Inner as ::sotu::blocking::IntoStateMachine>::Superstate<
                            '_,
                        > as ::sotu::blocking::Superstate<
                            inner::Inner,
                        >>::call_entry_action(state, submachine, context);
                    }
                }
            }
        }
        fn call_exit_action<'ctx, 'data: 'ctx>(
            &mut self,
            shared_storage: &mut Outer,
            context: &mut (),
        ) {
            match self {
                Superstate::Nested(::sotu::SubmachineNode::Mount) => {
                    ::sotu::private::MountField::<
                        inner::Inner,
                    >::release(&mut shared_storage.inner)
                }
                Superstate::Nested(::sotu::SubmachineNode::Inner(state)) => {
                    if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                        inner::Inner,
                    >::get_mut(&mut shared_storage.inner) {
                        <<inner::Inner as ::sotu::blocking::IntoStateMachine>::Superstate<
                            '_,
                        > as ::sotu::blocking::Superstate<
                            inner::Inner,
                        >>::call_exit_action(state, submachine, context);
                    }
                }
            }
        }
        fn superstate(&mut self) -> ::core::option::Option<Superstate<'_>> {
            match self {
                Superstate::Nested(::sotu::SubmachineNode::Mount) => {
                    ::core::option::Option::None
                }
                Superstate::Nested(::sotu::SubmachineNode::Inner(superstate)) => {
                    ::core::option::Option::Some(
                        Superstate::Nested(
                            match <<inner::Inner as ::sotu::blocking::IntoStateMachine>::Superstate<
                                '_,
                            > as ::sotu::blocking::Superstate<
                                inner::Inner,
                            >>::superstate(superstate) {
                                ::core::option::Option::Some(parent) => {
                                    ::sotu::SubmachineNode::Inner(parent)
                                }
                                ::core::option::Option::None => {
                                    ::sotu::SubmachineNode::Mount
                                }
                            },
                        ),
                    )
                }
            }
        }
        fn same_state(&self, other: &Superstate<'_>) -> bool {
            match (self, other) {
                (
                    Superstate::Nested(::sotu::SubmachineNode::Mount),
                    Superstate::Nested(::sotu::SubmachineNode::Mount),
                ) => true,
                (
                    Superstate::Nested(::sotu::SubmachineNode::Inner(first)),
                    Superstate::Nested(::sotu::SubmachineNode::Inner(second)),
                ) => {
                    <<inner::Inner as ::sotu::blocking::IntoStateMachine>::Superstate<
                        '_,
                    > as ::sotu::blocking::Superstate<
                        inner::Inner,
                    >>::same_state(first, second)
                }
                _ => false,
            }
        }
        fn __is_addressed(&self, event: ()) -> (bool, ()) {
            match self {
                Superstate::Nested(::sotu::SubmachineNode::Inner(node)) => {
                    match Outer::__project_into_nested(
                        event,
                        |event| {
                            <<inner::Inner as ::sotu::blocking::IntoStateMachine>::Superstate<
                                '_,
                            > as ::sotu::blocking::Superstate<
                                inner::Inner,
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
            shared_storage: &mut Outer,
            event: (),
            context: &mut (),
        ) -> ::sotu::private::Dispatched<State, ::core::convert::Infallible, ()> {
            match self {
                Superstate::Nested(::sotu::SubmachineNode::Inner(node)) => {
                    let mount_offset = <<inner::Inner as ::sotu::blocking::IntoStateMachine>::Superstate<
                        '_,
                    > as ::sotu::blocking::SuperstateExt<inner::Inner>>::depth(node);
                    let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                        inner::Inner,
                    >::get_mut(&mut shared_storage.inner) else {
                        return ::sotu::private::Dispatched {
                            outcome: ::sotu::private::NodeOutcome::DeferToMount {
                                mount_offset,
                            },
                            event: event,
                        };
                    };
                    let visited = Outer::__project_into_nested(
                        event,
                        |event| {
                            let ::sotu::private::Dispatched { outcome, event } = <<inner::Inner as ::sotu::blocking::IntoStateMachine>::Superstate<
                                '_,
                            > as ::sotu::blocking::Superstate<
                                inner::Inner,
                            >>::__dispatch(node, submachine, event, context);
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
                                            State::Nested(target),
                                        )
                                    }
                                    ::sotu::private::NodeOutcome::ExternalTransition {
                                        target,
                                        handler_offset,
                                    } => {
                                        ::sotu::private::NodeOutcome::ExternalTransition {
                                            target: State::Nested(target),
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
                        Outer,
                    >::call_handler(self, shared_storage, &event, context);
                    ::sotu::private::Dispatched {
                        outcome: ::sotu::private::NodeOutcome::from(outcome),
                        event: event,
                    }
                }
            }
        }
    }
    impl ::sotu::private::Submachine for Outer {
        type Event = ();
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            Superstate::nested()
        }
    }
    impl ::sotu::private::BlockingSubmachine for Outer {}
    impl ::sotu::private::ExitlessSubmachine for Outer {}
    impl Outer {
        #[doc(hidden)]
        #[allow(unreachable_code)]
        fn __project_into_nested<Visited>(
            event: (),
            visit: impl ::core::ops::FnOnce(
                <inner::Inner as ::sotu::private::Submachine>::Event,
            ) -> (Visited, <inner::Inner as ::sotu::private::Submachine>::Event),
        ) -> ::core::result::Result<(Visited, ()), ()> {
            if !<<inner::Inner as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
                (),
            >>::accepts(&event) {
                return ::core::result::Result::Err(event);
            }
            match <<inner::Inner as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
                (),
            >>::project(event) {
                ::core::result::Result::Ok(child_event) => {
                    let (visited, child_event) = visit(child_event);
                    ::core::result::Result::Ok((
                        visited,
                        <<inner::Inner as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
                            (),
                        >>::embed(child_event),
                    ))
                }
                ::core::result::Result::Err(event) => ::core::result::Result::Err(event),
            }
        }
    }
}
pub struct Nest {
    outer: Option<outer::Outer>,
}
#[automatically_derived]
impl ::core::default::Default for Nest {
    #[inline]
    fn default() -> Nest {
        Nest {
            outer: ::core::default::Default::default(),
        }
    }
}
impl Nest {
    fn holding() -> Outcome<State> {
        Handled
    }
}
pub enum State {
    Holding(<outer::Outer as ::sotu::private::Submachine>::State),
}
impl State {
    pub fn holding(state: <outer::Outer as ::sotu::private::Submachine>::State) -> Self {
        Self::Holding(state)
    }
}
pub enum Superstate<'sub> {
    Holding(
        ::sotu::SubmachineNode<
            <outer::Outer as ::sotu::private::Submachine>::Superstate<'sub>,
        >,
    ),
}
impl Superstate<'_> {
    pub fn holding() -> State {
        State::Holding(<outer::Outer as ::sotu::private::Submachine>::initial())
    }
}
impl ::sotu::blocking::IntoStateMachine for Nest {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        Superstate::holding()
    }
}
impl ::sotu::blocking::State<Nest> for State
where
    outer::Outer: ::sotu::private::BlockingSubmachine,
    outer::Outer: ::sotu::private::ExitlessSubmachine,
{
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Nest,
        _event: &(),
        _context: &mut (),
    ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
        match self {
            State::Holding(_) => ::sotu::Outcome::Super,
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Nest,
        context: &mut (),
    ) {
        match self {
            State::Holding(state) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    outer::Outer,
                >::get_mut(&mut shared_storage.outer) {
                    <<outer::Outer as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                        outer::Outer,
                    >>::call_entry_action(state, submachine, context);
                }
            }
        }
    }
    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Nest,
        context: &mut (),
    ) {
        match self {
            State::Holding(state) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    outer::Outer,
                >::get_mut(&mut shared_storage.outer) {
                    <<outer::Outer as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                        outer::Outer,
                    >>::call_exit_action(state, submachine, context);
                }
            }
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate<'_>> {
        match self {
            State::Holding(state) => {
                ::core::option::Option::Some(
                    Superstate::Holding(
                        match <<outer::Outer as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            outer::Outer,
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
            (State::Holding(first), State::Holding(second)) => {
                <<outer::Outer as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                    outer::Outer,
                >>::same_state(first, second)
            }
        }
    }
    fn __is_addressed(&self, event: ()) -> (bool, ()) {
        match self {
            State::Holding(node) => {
                match Nest::__project_into_holding(
                    event,
                    |event| {
                        <<outer::Outer as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            outer::Outer,
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
        shared_storage: &mut Nest,
        event: (),
        context: &mut (),
    ) -> ::sotu::private::Dispatched<Self, ::core::convert::Infallible, ()> {
        match self {
            State::Holding(node) => {
                let mount_offset = <<outer::Outer as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::StateExt<
                    outer::Outer,
                >>::depth(node);
                let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    outer::Outer,
                >::get_mut(&mut shared_storage.outer) else {
                    return ::sotu::private::Dispatched {
                        outcome: ::sotu::private::NodeOutcome::DeferToMount {
                            mount_offset,
                        },
                        event: event,
                    };
                };
                let visited = Nest::__project_into_holding(
                    event,
                    |event| {
                        let ::sotu::private::Dispatched { outcome, event } = <<outer::Outer as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            outer::Outer,
                        >>::__dispatch(node, submachine, event, context);
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
                                        State::Holding(target),
                                    )
                                }
                                ::sotu::private::NodeOutcome::ExternalTransition {
                                    target,
                                    handler_offset,
                                } => {
                                    ::sotu::private::NodeOutcome::ExternalTransition {
                                        target: State::Holding(target),
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
impl<'sub> ::sotu::blocking::Superstate<Nest> for Superstate<'sub>
where
    outer::Outer: ::sotu::private::BlockingSubmachine,
    outer::Outer: ::sotu::private::ExitlessSubmachine,
{
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Nest,
        _event: &(),
        _context: &mut (),
    ) -> ::sotu::Outcome<State, ::core::convert::Infallible> {
        match self {
            Superstate::Holding(::sotu::SubmachineNode::Inner(_)) => {
                ::sotu::Outcome::Super
            }
            Superstate::Holding(::sotu::SubmachineNode::Mount) => Nest::holding(),
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Nest,
        context: &mut (),
    ) {
        match self {
            Superstate::Holding(::sotu::SubmachineNode::Mount) => {
                ::sotu::private::DefaultInstance::<
                    outer::Outer,
                >::fill_default(&mut shared_storage.outer)
            }
            Superstate::Holding(::sotu::SubmachineNode::Inner(state)) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    outer::Outer,
                >::get_mut(&mut shared_storage.outer) {
                    <<outer::Outer as ::sotu::blocking::IntoStateMachine>::Superstate<
                        '_,
                    > as ::sotu::blocking::Superstate<
                        outer::Outer,
                    >>::call_entry_action(state, submachine, context);
                }
            }
        }
    }
    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Nest,
        context: &mut (),
    ) {
        match self {
            Superstate::Holding(::sotu::SubmachineNode::Mount) => {
                ::sotu::private::MountField::<
                    outer::Outer,
                >::release(&mut shared_storage.outer)
            }
            Superstate::Holding(::sotu::SubmachineNode::Inner(state)) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    outer::Outer,
                >::get_mut(&mut shared_storage.outer) {
                    <<outer::Outer as ::sotu::blocking::IntoStateMachine>::Superstate<
                        '_,
                    > as ::sotu::blocking::Superstate<
                        outer::Outer,
                    >>::call_exit_action(state, submachine, context);
                }
            }
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate<'_>> {
        match self {
            Superstate::Holding(::sotu::SubmachineNode::Mount) => {
                ::core::option::Option::None
            }
            Superstate::Holding(::sotu::SubmachineNode::Inner(superstate)) => {
                ::core::option::Option::Some(
                    Superstate::Holding(
                        match <<outer::Outer as ::sotu::blocking::IntoStateMachine>::Superstate<
                            '_,
                        > as ::sotu::blocking::Superstate<
                            outer::Outer,
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
                Superstate::Holding(::sotu::SubmachineNode::Mount),
                Superstate::Holding(::sotu::SubmachineNode::Mount),
            ) => true,
            (
                Superstate::Holding(::sotu::SubmachineNode::Inner(first)),
                Superstate::Holding(::sotu::SubmachineNode::Inner(second)),
            ) => {
                <<outer::Outer as ::sotu::blocking::IntoStateMachine>::Superstate<
                    '_,
                > as ::sotu::blocking::Superstate<
                    outer::Outer,
                >>::same_state(first, second)
            }
            _ => false,
        }
    }
    fn __is_addressed(&self, event: ()) -> (bool, ()) {
        match self {
            Superstate::Holding(::sotu::SubmachineNode::Inner(node)) => {
                match Nest::__project_into_holding(
                    event,
                    |event| {
                        <<outer::Outer as ::sotu::blocking::IntoStateMachine>::Superstate<
                            '_,
                        > as ::sotu::blocking::Superstate<
                            outer::Outer,
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
        shared_storage: &mut Nest,
        event: (),
        context: &mut (),
    ) -> ::sotu::private::Dispatched<State, ::core::convert::Infallible, ()> {
        match self {
            Superstate::Holding(::sotu::SubmachineNode::Inner(node)) => {
                let mount_offset = <<outer::Outer as ::sotu::blocking::IntoStateMachine>::Superstate<
                    '_,
                > as ::sotu::blocking::SuperstateExt<outer::Outer>>::depth(node);
                let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    outer::Outer,
                >::get_mut(&mut shared_storage.outer) else {
                    return ::sotu::private::Dispatched {
                        outcome: ::sotu::private::NodeOutcome::DeferToMount {
                            mount_offset,
                        },
                        event: event,
                    };
                };
                let visited = Nest::__project_into_holding(
                    event,
                    |event| {
                        let ::sotu::private::Dispatched { outcome, event } = <<outer::Outer as ::sotu::blocking::IntoStateMachine>::Superstate<
                            '_,
                        > as ::sotu::blocking::Superstate<
                            outer::Outer,
                        >>::__dispatch(node, submachine, event, context);
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
                                        State::Holding(target),
                                    )
                                }
                                ::sotu::private::NodeOutcome::ExternalTransition {
                                    target,
                                    handler_offset,
                                } => {
                                    ::sotu::private::NodeOutcome::ExternalTransition {
                                        target: State::Holding(target),
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
                    Nest,
                >::call_handler(self, shared_storage, &event, context);
                ::sotu::private::Dispatched {
                    outcome: ::sotu::private::NodeOutcome::from(outcome),
                    event: event,
                }
            }
        }
    }
}
impl ::sotu::private::Submachine for Nest {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        Superstate::holding()
    }
}
impl ::sotu::private::BlockingSubmachine for Nest {}
impl ::sotu::private::ExitlessSubmachine for Nest {}
impl Nest {
    #[doc(hidden)]
    #[allow(unreachable_code)]
    fn __project_into_holding<Visited>(
        event: (),
        visit: impl ::core::ops::FnOnce(
            <outer::Outer as ::sotu::private::Submachine>::Event,
        ) -> (Visited, <outer::Outer as ::sotu::private::Submachine>::Event),
    ) -> ::core::result::Result<(Visited, ()), ()> {
        if !<<outer::Outer as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
            (),
        >>::accepts(&event) {
            return ::core::result::Result::Err(event);
        }
        match <<outer::Outer as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
            (),
        >>::project(event) {
            ::core::result::Result::Ok(child_event) => {
                let (visited, child_event) = visit(child_event);
                ::core::result::Result::Ok((
                    visited,
                    <<outer::Outer as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
                        (),
                    >>::embed(child_event),
                ))
            }
            ::core::result::Result::Err(event) => ::core::result::Result::Err(event),
        }
    }
}
fn main() {
    let mut machine = Nest::default().state_machine();
    machine.step(&mut ());
}
