use sotu::prelude::*;
mod borrowing {
    use sotu::prelude::*;
    pub struct Heater {
        pub seen: u32,
    }
    #[automatically_derived]
    impl ::core::default::Default for Heater {
        #[inline]
        fn default() -> Heater {
            Heater {
                seen: ::core::default::Default::default(),
            }
        }
    }
    impl Heater {
        fn heating() -> Outcome<State> {
            Super
        }
        fn cold() -> Outcome<State> {
            Handled
        }
        fn powered(level: &mut u32) -> Outcome<State> {
            *level += 1;
            Transition(State::cold())
        }
        fn power_on(&mut self, level: &mut u32) {
            *level += 1;
            self.seen = *level;
        }
        fn power_off(&mut self, level: &u32) {
            self.seen = *level;
        }
    }
    pub enum State {
        Heating { level: u32, spare: u8 },
        Cold {},
    }
    impl State {
        pub fn heating(level: u32, spare: u8) -> Self {
            Self::Heating { level, spare }
        }
        pub fn cold() -> Self {
            Self::Cold {}
        }
    }
    pub enum Superstate<'sub> {
        Powered { level: &'sub mut u32 },
    }
    impl ::sotu::blocking::IntoStateMachine for Heater {
        type Event = ();
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::heating(0, 0)
        }
    }
    impl ::sotu::blocking::State<Heater> for State {
        fn call_handler<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Heater,
            _event: &(),
            _context: &mut (),
        ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
            match self {
                State::Heating { .. } => Heater::heating(),
                State::Cold {} => Heater::cold(),
            }
        }
        fn superstate(&mut self) -> ::core::option::Option<Superstate<'_>> {
            match self {
                State::Heating { level, .. } => {
                    ::core::option::Option::Some(Superstate::Powered { level })
                }
                State::Cold {} => ::core::option::Option::None,
            }
        }
    }
    impl<'sub> ::sotu::blocking::Superstate<Heater> for Superstate<'sub> {
        fn call_handler<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Heater,
            _event: &(),
            _context: &mut (),
        ) -> ::sotu::Outcome<State, ::core::convert::Infallible> {
            match self {
                Superstate::Powered { level } => Heater::powered(level),
            }
        }
        fn call_entry_action<'ctx, 'data: 'ctx>(
            &mut self,
            shared_storage: &mut Heater,
            _context: &mut (),
        ) {
            match self {
                Superstate::Powered { level } => Heater::power_on(shared_storage, level),
            }
        }
        fn call_exit_action<'ctx, 'data: 'ctx>(
            &mut self,
            shared_storage: &mut Heater,
            _context: &mut (),
        ) {
            match self {
                Superstate::Powered { level } => Heater::power_off(shared_storage, level),
            }
        }
        fn superstate(&mut self) -> ::core::option::Option<Superstate<'_>> {
            match self {
                Superstate::Powered { .. } => ::core::option::Option::None,
            }
        }
        fn same_state(&self, other: &Superstate<'_>) -> bool {
            match (self, other) {
                (Superstate::Powered { .. }, Superstate::Powered { .. }) => true,
            }
        }
    }
    impl ::sotu::private::Submachine for Heater {
        type Event = ();
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::heating(0, 0)
        }
    }
    impl ::sotu::private::BlockingSubmachine for Heater {}
}
mod field_less {
    use sotu::prelude::*;
    pub struct Fan {
        pub starts: u32,
    }
    #[automatically_derived]
    impl ::core::default::Default for Fan {
        #[inline]
        fn default() -> Fan {
            Fan {
                starts: ::core::default::Default::default(),
            }
        }
    }
    impl Fan {
        fn spinning() -> Outcome<State> {
            Super
        }
        fn running() -> Outcome<State> {
            Super
        }
        fn powered() -> Outcome<State> {
            Handled
        }
        fn start(&mut self) {
            self.starts += 1;
        }
        fn stop() {}
    }
    pub enum State {
        Spinning {},
    }
    impl State {
        pub fn spinning() -> Self {
            Self::Spinning {}
        }
    }
    pub enum Superstate {
        Running {},
        Powered {},
    }
    impl ::sotu::blocking::IntoStateMachine for Fan {
        type Event = ();
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = Superstate where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::spinning()
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
                State::Spinning {} => Fan::spinning(),
            }
        }
        fn superstate(&mut self) -> ::core::option::Option<Superstate> {
            match self {
                State::Spinning {} => {
                    ::core::option::Option::Some(Superstate::Running {})
                }
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
                Superstate::Running {} => Fan::running(),
                Superstate::Powered {} => Fan::powered(),
            }
        }
        fn call_entry_action<'ctx, 'data: 'ctx>(
            &mut self,
            shared_storage: &mut Fan,
            _context: &mut (),
        ) {
            match self {
                Superstate::Running {} => {}
                Superstate::Powered {} => Fan::start(shared_storage),
            }
        }
        fn call_exit_action<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Fan,
            _context: &mut (),
        ) {
            match self {
                Superstate::Running {} => Fan::stop(),
                Superstate::Powered {} => {}
            }
        }
        fn superstate(&mut self) -> ::core::option::Option<Superstate> {
            match self {
                Superstate::Running {} => {
                    ::core::option::Option::Some(Superstate::Powered {})
                }
                Superstate::Powered {} => ::core::option::Option::None,
            }
        }
        fn same_state(&self, other: &Superstate) -> bool {
            match (self, other) {
                (Superstate::Running { .. }, Superstate::Running { .. }) => true,
                (Superstate::Powered { .. }, Superstate::Powered { .. }) => true,
                _ => false,
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
            State::spinning()
        }
    }
    impl ::sotu::private::BlockingSubmachine for Fan {}
}
fn main() {
    let mut heater = borrowing::Heater::default().state_machine();
    let mut fan = field_less::Fan::default().state_machine();
    heater.step(&mut ());
    fan.step(&mut ());
}
