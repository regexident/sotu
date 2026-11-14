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
    impl Door {
        fn closed(context: &mut u8) -> Outcome<State> {
            *context += 1;
            Handled
        }
    }
    pub enum State {
        Closed {},
    }
    impl State {
        pub fn closed() -> Self {
            Self::Closed {}
        }
    }
    impl ::sotu::blocking::IntoStateMachine for Door {
        type Event = ();
        type Context<'ctx, 'data: 'ctx> = u8;
        type State = State;
        type Superstate<'sub> = () where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::closed()
        }
    }
    impl ::sotu::blocking::State<Door> for State {
        fn call_handler<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Door,
            _event: &(),
            context: &mut u8,
        ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
            match self {
                State::Closed {} => Door::closed(context),
            }
        }
    }
    impl ::sotu::private::Submachine for Door {
        type Event = ();
        type Context<'ctx, 'data: 'ctx> = u8;
        type State = State;
        type Superstate<'sub> = () where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::closed()
        }
    }
    impl ::sotu::private::BlockingSubmachine for Door {}
    impl ::sotu::private::ExitlessSubmachine for Door {}
}
pub struct House {
    door: Option<door::Door>,
}
#[automatically_derived]
impl ::core::default::Default for House {
    #[inline]
    fn default() -> House {
        House {
            door: ::core::default::Default::default(),
        }
    }
}
impl House {
    fn idle(context: &mut u16) -> Outcome<State> {
        *context += 1;
        Transition(Superstate::loading())
    }
    fn loading() -> Outcome<State> {
        Handled
    }
}
pub enum State {
    Idle {},
    Loading(<door::Door as ::sotu::private::Submachine>::State),
}
impl State {
    pub fn idle() -> Self {
        Self::Idle {}
    }
    pub fn loading(state: <door::Door as ::sotu::private::Submachine>::State) -> Self {
        Self::Loading(state)
    }
}
pub enum Superstate<'sub> {
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
impl ::sotu::blocking::IntoStateMachine for House {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = u16;
    type State = State;
    type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::idle()
    }
}
impl ::sotu::blocking::State<House> for State
where
    door::Door: ::sotu::private::BlockingSubmachine,
    door::Door: ::sotu::private::ExitlessSubmachine,
{
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut House,
        _event: &(),
        context: &mut u16,
    ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
        match self {
            State::Idle {} => House::idle(context),
            State::Loading(_) => ::sotu::Outcome::Super,
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut House,
        context: &mut u16,
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
        shared_storage: &mut House,
        context: &mut u16,
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
            State::Idle {} => ::core::option::Option::None,
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
}
impl<'sub> ::sotu::blocking::Superstate<House> for Superstate<'sub>
where
    door::Door: ::sotu::private::BlockingSubmachine,
    door::Door: ::sotu::private::ExitlessSubmachine,
{
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut House,
        _event: &(),
        _context: &mut u16,
    ) -> ::sotu::Outcome<State, ::core::convert::Infallible> {
        match self {
            Superstate::Loading(::sotu::SubmachineNode::Inner(_)) => {
                ::sotu::Outcome::Super
            }
            Superstate::Loading(::sotu::SubmachineNode::Mount) => House::loading(),
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut House,
        context: &mut u16,
    ) {
        match self {
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
        shared_storage: &mut House,
        context: &mut u16,
    ) {
        match self {
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
            Superstate::Loading(::sotu::SubmachineNode::Mount) => {
                ::core::option::Option::None
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
}
impl ::sotu::private::Submachine for House {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = u16;
    type State = State;
    type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::idle()
    }
}
impl ::sotu::private::BlockingSubmachine for House {}
impl ::sotu::private::ExitlessSubmachine for House {}
fn main() {}
