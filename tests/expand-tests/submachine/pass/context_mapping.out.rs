use sotu::prelude::*;
mod printer {
    use sotu::prelude::*;
    /// The context of the printer, borrowed from the parent's context.
    pub struct Sink<'ctx, 'data> {
        pub lines: &'ctx mut Vec<&'data str>,
    }
    pub struct Printer;
    #[automatically_derived]
    impl ::core::default::Default for Printer {
        #[inline]
        fn default() -> Printer {
            Printer {}
        }
    }
    impl Printer {
        fn before(
            &mut self,
            _node: StateOrSuperstate<'_, State, Superstate>,
            _event: &(),
            context: &mut Sink<'_, '_>,
        ) {
            context.lines.push("dispatch");
        }
        fn after(
            &mut self,
            _source: &State,
            _target: &State,
            context: &mut Sink<'_, '_>,
        ) {
            context.lines.push("transition");
        }
        fn idle() -> Outcome<State> {
            Transition(State::printing())
        }
        fn printing(context: &mut Sink<'_, '_>) -> Outcome<State> {
            context.lines.push("page");
            Super
        }
        fn powered() -> Outcome<State> {
            Handled
        }
        fn warm(context: &mut Sink<'_, '_>) {
            context.lines.push("warm");
        }
    }
    pub enum State {
        Idle {},
        Printing {},
    }
    impl State {
        pub fn idle() -> Self {
            Self::Idle {}
        }
        pub fn printing() -> Self {
            Self::Printing {}
        }
    }
    pub enum Superstate {
        Powered {},
    }
    impl ::sotu::blocking::IntoStateMachine for Printer {
        type Event = ();
        type Context<'ctx, 'data: 'ctx> = Sink<'ctx, 'data>;
        type State = State;
        type Superstate<'sub> = Superstate where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::idle()
        }
        fn before_dispatch<'ctx, 'data: 'ctx>(
            &mut self,
            node: ::sotu::StateOrSuperstate<'_, State, Superstate>,
            event: &(),
            context: &mut Sink<'ctx, 'data>,
        ) {
            ::sotu::blocking::DispatchHook::call_dispatch_hook(
                &(Self::before),
                self,
                node,
                event,
                context,
            )
        }
        fn after_transition<'ctx, 'data: 'ctx>(
            &mut self,
            source: &State,
            target: &State,
            context: &mut Sink<'ctx, 'data>,
        ) {
            ::sotu::blocking::TransitionHook::call_transition_hook(
                &(Self::after),
                self,
                source,
                target,
                context,
            )
        }
    }
    impl ::sotu::blocking::State<Printer> for State {
        fn call_handler<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Printer,
            _event: &(),
            context: &mut Sink<'ctx, 'data>,
        ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
            match self {
                State::Idle {} => Printer::idle(),
                State::Printing {} => Printer::printing(context),
            }
        }
        fn call_entry_action<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Printer,
            context: &mut Sink<'ctx, 'data>,
        ) {
            match self {
                State::Idle {} => Printer::warm(context),
                State::Printing {} => {}
            }
        }
        fn superstate(&mut self) -> ::core::option::Option<Superstate> {
            match self {
                State::Idle {} => ::core::option::Option::Some(Superstate::Powered {}),
                State::Printing {} => {
                    ::core::option::Option::Some(Superstate::Powered {})
                }
            }
        }
    }
    impl ::sotu::blocking::Superstate<Printer> for Superstate {
        fn call_handler<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Printer,
            _event: &(),
            _context: &mut Sink<'ctx, 'data>,
        ) -> ::sotu::Outcome<State, ::core::convert::Infallible> {
            match self {
                Superstate::Powered {} => Printer::powered(),
            }
        }
        fn superstate(&mut self) -> ::core::option::Option<Superstate> {
            match self {
                Superstate::Powered {} => ::core::option::Option::None,
            }
        }
        fn same_state(&self, other: &Superstate) -> bool {
            match (self, other) {
                (Superstate::Powered { .. }, Superstate::Powered { .. }) => true,
            }
        }
    }
    impl ::sotu::private::Submachine for Printer {
        type Event = ();
        type Context<'ctx, 'data: 'ctx> = Sink<'ctx, 'data>;
        type State = State;
        type Superstate<'sub> = Superstate where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::idle()
        }
    }
    impl ::sotu::private::BlockingSubmachine for Printer {}
    impl ::sotu::private::ExitlessSubmachine for Printer {}
}
/// The context of the office, which the printer borrows for each callback.
pub struct Writer<'ctx, 'data> {
    lines: &'ctx mut Vec<&'data str>,
}
fn map_context<'a, 'ctx, 'data>(
    writer: &'a mut Writer<'ctx, 'data>,
) -> printer::Sink<'a, 'data> {
    printer::Sink {
        lines: &mut *writer.lines,
    }
}
pub struct Office {
    printer: printer::Printer,
}
#[automatically_derived]
impl ::core::default::Default for Office {
    #[inline]
    fn default() -> Office {
        Office {
            printer: ::core::default::Default::default(),
        }
    }
}
impl Office {
    fn printing() -> Outcome<State> {
        Handled
    }
}
pub enum State {
    Printing(<printer::Printer as ::sotu::private::Submachine>::State),
}
impl State {
    pub fn printing(
        state: <printer::Printer as ::sotu::private::Submachine>::State,
    ) -> Self {
        Self::Printing(state)
    }
}
pub enum Superstate<'sub> {
    Printing(
        ::sotu::SubmachineNode<
            <printer::Printer as ::sotu::private::Submachine>::Superstate<'sub>,
        >,
    ),
}
impl Superstate<'_> {
    pub fn printing() -> State {
        State::Printing(<printer::Printer as ::sotu::private::Submachine>::initial())
    }
}
impl ::sotu::blocking::IntoStateMachine for Office {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = Writer<'ctx, 'data>;
    type State = State;
    type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        Superstate::printing()
    }
}
impl ::sotu::blocking::State<Office> for State
where
    printer::Printer: ::sotu::private::BlockingSubmachine,
    printer::Printer: ::sotu::private::ExitlessSubmachine,
{
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Office,
        _event: &(),
        _context: &mut Writer<'ctx, 'data>,
    ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
        match self {
            State::Printing(_) => ::sotu::Outcome::Super,
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Office,
        context: &mut Writer<'ctx, 'data>,
    ) {
        match self {
            State::Printing(state) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    printer::Printer,
                >::get_mut(&mut shared_storage.printer) {
                    {
                        let mut child_context = (map_context)(context);
                        <<printer::Printer as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            printer::Printer,
                        >>::call_entry_action(state, submachine, &mut child_context)
                    };
                }
            }
        }
    }
    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Office,
        context: &mut Writer<'ctx, 'data>,
    ) {
        match self {
            State::Printing(state) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    printer::Printer,
                >::get_mut(&mut shared_storage.printer) {
                    {
                        let mut child_context = (map_context)(context);
                        <<printer::Printer as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            printer::Printer,
                        >>::call_exit_action(state, submachine, &mut child_context)
                    };
                }
            }
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate<'_>> {
        match self {
            State::Printing(state) => {
                ::core::option::Option::Some(
                    Superstate::Printing(
                        match <<printer::Printer as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            printer::Printer,
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
            (State::Printing(first), State::Printing(second)) => {
                <<printer::Printer as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                    printer::Printer,
                >>::same_state(first, second)
            }
        }
    }
    fn __is_addressed(&self, event: ()) -> (bool, ()) {
        match self {
            State::Printing(node) => {
                match Office::__project_into_printing(
                    event,
                    |event| {
                        <<printer::Printer as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            printer::Printer,
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
        shared_storage: &mut Office,
        event: (),
        context: &mut Writer<'ctx, 'data>,
    ) -> ::sotu::private::Dispatched<Self, ::core::convert::Infallible, ()> {
        match self {
            State::Printing(node) => {
                let mount_offset = <<printer::Printer as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::StateExt<
                    printer::Printer,
                >>::depth(node);
                let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    printer::Printer,
                >::get_mut(&mut shared_storage.printer) else {
                    return ::sotu::private::Dispatched {
                        outcome: ::sotu::private::NodeOutcome::DeferToMount {
                            mount_offset,
                        },
                        event: event,
                    };
                };
                let visited = Office::__project_into_printing(
                    event,
                    |event| {
                        {
                            let mut child_context = (map_context)(context);
                            <printer::Printer as ::sotu::blocking::IntoStateMachine>::before_dispatch(
                                submachine,
                                ::sotu::StateOrSuperstate::State(&*node),
                                &event,
                                &mut child_context,
                            )
                        };
                        let ::sotu::private::Dispatched { outcome, event } = {
                            let mut child_context = (map_context)(context);
                            <<printer::Printer as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                                printer::Printer,
                            >>::__dispatch(node, submachine, event, &mut child_context)
                        };
                        {
                            let mut child_context = (map_context)(context);
                            <printer::Printer as ::sotu::blocking::IntoStateMachine>::after_dispatch(
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
                                        State::Printing(target),
                                    )
                                }
                                ::sotu::private::NodeOutcome::ExternalTransition {
                                    target,
                                    handler_offset,
                                } => {
                                    ::sotu::private::NodeOutcome::ExternalTransition {
                                        target: State::Printing(target),
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
            (State::Printing(source), State::Printing(target)) => {
                if exit_levels
                    <= <<printer::Printer as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::StateExt<
                        printer::Printer,
                    >>::depth(source)
                {
                    1
                        + <<printer::Printer as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                            printer::Printer,
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
        shared_storage: &mut Office,
        context: &mut Writer<'ctx, 'data>,
    ) {
        match (self, target) {
            (State::Printing(source), State::Printing(target)) => {
                if exit_levels
                    > <<printer::Printer as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::StateExt<
                        printer::Printer,
                    >>::depth(source)
                {
                    return;
                }
                let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    printer::Printer,
                >::get_mut(&mut shared_storage.printer) else {
                    return;
                };
                match nesting.checked_sub(1) {
                    ::core::option::Option::Some(0) => {
                        {
                            let mut child_context = (map_context)(context);
                            <printer::Printer as ::sotu::blocking::IntoStateMachine>::before_transition(
                                submachine,
                                source,
                                target,
                                &mut child_context,
                            )
                        };
                    }
                    ::core::option::Option::Some(deeper) => {
                        {
                            let mut child_context = (map_context)(context);
                            <<printer::Printer as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                                printer::Printer,
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
        shared_storage: &mut Office,
        context: &mut Writer<'ctx, 'data>,
    ) {
        match (self, target) {
            (State::Printing(source), State::Printing(target)) => {
                if exit_levels
                    > <<printer::Printer as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::StateExt<
                        printer::Printer,
                    >>::depth(source)
                {
                    return;
                }
                let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    printer::Printer,
                >::get_mut(&mut shared_storage.printer) else {
                    return;
                };
                match nesting.checked_sub(1) {
                    ::core::option::Option::Some(0) => {
                        {
                            let mut child_context = (map_context)(context);
                            <printer::Printer as ::sotu::blocking::IntoStateMachine>::after_transition(
                                submachine,
                                source,
                                target,
                                &mut child_context,
                            )
                        };
                    }
                    ::core::option::Option::Some(deeper) => {
                        {
                            let mut child_context = (map_context)(context);
                            <<printer::Printer as ::sotu::blocking::IntoStateMachine>::State as ::sotu::blocking::State<
                                printer::Printer,
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
impl<'sub> ::sotu::blocking::Superstate<Office> for Superstate<'sub>
where
    printer::Printer: ::sotu::private::BlockingSubmachine,
    printer::Printer: ::sotu::private::ExitlessSubmachine,
{
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Office,
        _event: &(),
        _context: &mut Writer<'ctx, 'data>,
    ) -> ::sotu::Outcome<State, ::core::convert::Infallible> {
        match self {
            Superstate::Printing(::sotu::SubmachineNode::Inner(_)) => {
                ::sotu::Outcome::Super
            }
            Superstate::Printing(::sotu::SubmachineNode::Mount) => Office::printing(),
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Office,
        context: &mut Writer<'ctx, 'data>,
    ) {
        match self {
            Superstate::Printing(::sotu::SubmachineNode::Mount) => {
                ::sotu::private::DefaultInstance::<
                    printer::Printer,
                >::fill_default(&mut shared_storage.printer)
            }
            Superstate::Printing(::sotu::SubmachineNode::Inner(state)) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    printer::Printer,
                >::get_mut(&mut shared_storage.printer) {
                    {
                        let mut child_context = (map_context)(context);
                        <<printer::Printer as ::sotu::blocking::IntoStateMachine>::Superstate<
                            '_,
                        > as ::sotu::blocking::Superstate<
                            printer::Printer,
                        >>::call_entry_action(state, submachine, &mut child_context)
                    };
                }
            }
        }
    }
    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Office,
        context: &mut Writer<'ctx, 'data>,
    ) {
        match self {
            Superstate::Printing(::sotu::SubmachineNode::Mount) => {
                ::sotu::private::MountField::<
                    printer::Printer,
                >::release(&mut shared_storage.printer)
            }
            Superstate::Printing(::sotu::SubmachineNode::Inner(state)) => {
                if let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    printer::Printer,
                >::get_mut(&mut shared_storage.printer) {
                    {
                        let mut child_context = (map_context)(context);
                        <<printer::Printer as ::sotu::blocking::IntoStateMachine>::Superstate<
                            '_,
                        > as ::sotu::blocking::Superstate<
                            printer::Printer,
                        >>::call_exit_action(state, submachine, &mut child_context)
                    };
                }
            }
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate<'_>> {
        match self {
            Superstate::Printing(::sotu::SubmachineNode::Mount) => {
                ::core::option::Option::None
            }
            Superstate::Printing(::sotu::SubmachineNode::Inner(superstate)) => {
                ::core::option::Option::Some(
                    Superstate::Printing(
                        match <<printer::Printer as ::sotu::blocking::IntoStateMachine>::Superstate<
                            '_,
                        > as ::sotu::blocking::Superstate<
                            printer::Printer,
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
                Superstate::Printing(::sotu::SubmachineNode::Mount),
                Superstate::Printing(::sotu::SubmachineNode::Mount),
            ) => true,
            (
                Superstate::Printing(::sotu::SubmachineNode::Inner(first)),
                Superstate::Printing(::sotu::SubmachineNode::Inner(second)),
            ) => {
                <<printer::Printer as ::sotu::blocking::IntoStateMachine>::Superstate<
                    '_,
                > as ::sotu::blocking::Superstate<
                    printer::Printer,
                >>::same_state(first, second)
            }
            _ => false,
        }
    }
    fn __is_addressed(&self, event: ()) -> (bool, ()) {
        match self {
            Superstate::Printing(::sotu::SubmachineNode::Inner(node)) => {
                match Office::__project_into_printing(
                    event,
                    |event| {
                        <<printer::Printer as ::sotu::blocking::IntoStateMachine>::Superstate<
                            '_,
                        > as ::sotu::blocking::Superstate<
                            printer::Printer,
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
        shared_storage: &mut Office,
        event: (),
        context: &mut Writer<'ctx, 'data>,
    ) -> ::sotu::private::Dispatched<State, ::core::convert::Infallible, ()> {
        match self {
            Superstate::Printing(::sotu::SubmachineNode::Inner(node)) => {
                let mount_offset = <<printer::Printer as ::sotu::blocking::IntoStateMachine>::Superstate<
                    '_,
                > as ::sotu::blocking::SuperstateExt<printer::Printer>>::depth(node);
                let ::core::option::Option::Some(submachine) = ::sotu::private::MountField::<
                    printer::Printer,
                >::get_mut(&mut shared_storage.printer) else {
                    return ::sotu::private::Dispatched {
                        outcome: ::sotu::private::NodeOutcome::DeferToMount {
                            mount_offset,
                        },
                        event: event,
                    };
                };
                let visited = Office::__project_into_printing(
                    event,
                    |event| {
                        {
                            let mut child_context = (map_context)(context);
                            <printer::Printer as ::sotu::blocking::IntoStateMachine>::before_dispatch(
                                submachine,
                                ::sotu::StateOrSuperstate::Superstate(&*node),
                                &event,
                                &mut child_context,
                            )
                        };
                        let ::sotu::private::Dispatched { outcome, event } = {
                            let mut child_context = (map_context)(context);
                            <<printer::Printer as ::sotu::blocking::IntoStateMachine>::Superstate<
                                '_,
                            > as ::sotu::blocking::Superstate<
                                printer::Printer,
                            >>::__dispatch(node, submachine, event, &mut child_context)
                        };
                        {
                            let mut child_context = (map_context)(context);
                            <printer::Printer as ::sotu::blocking::IntoStateMachine>::after_dispatch(
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
                                        State::Printing(target),
                                    )
                                }
                                ::sotu::private::NodeOutcome::ExternalTransition {
                                    target,
                                    handler_offset,
                                } => {
                                    ::sotu::private::NodeOutcome::ExternalTransition {
                                        target: State::Printing(target),
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
                    Office,
                >::call_handler(self, shared_storage, &event, context);
                ::sotu::private::Dispatched {
                    outcome: ::sotu::private::NodeOutcome::from(outcome),
                    event: event,
                }
            }
        }
    }
}
impl ::sotu::private::Submachine for Office {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = Writer<'ctx, 'data>;
    type State = State;
    type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        Superstate::printing()
    }
}
impl ::sotu::private::BlockingSubmachine for Office {}
impl ::sotu::private::ExitlessSubmachine for Office {}
impl Office {
    #[doc(hidden)]
    #[allow(unreachable_code)]
    fn __project_into_printing<Visited>(
        event: (),
        visit: impl ::core::ops::FnOnce(
            <printer::Printer as ::sotu::private::Submachine>::Event,
        ) -> (Visited, <printer::Printer as ::sotu::private::Submachine>::Event),
    ) -> ::core::result::Result<(Visited, ()), ()> {
        if !<<printer::Printer as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
            (),
        >>::accepts(&event) {
            return ::core::result::Result::Err(event);
        }
        match <<printer::Printer as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
            (),
        >>::project(event) {
            ::core::result::Result::Ok(child_event) => {
                let (visited, child_event) = visit(child_event);
                ::core::result::Result::Ok((
                    visited,
                    <<printer::Printer as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<
                        (),
                    >>::embed(child_event),
                ))
            }
            ::core::result::Result::Err(event) => ::core::result::Result::Err(event),
        }
    }
}
fn main() {
    let mut lines = Vec::new();
    let mut machine = Office::default().state_machine();
    for _ in 0..2 {
        let mut writer = Writer { lines: &mut lines };
        machine.step(&mut writer);
    }
}
