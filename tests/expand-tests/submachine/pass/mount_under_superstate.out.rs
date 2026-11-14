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
fn main() {
    let mut machine = Room::default().state_machine();
    machine.step(&mut ());
}
