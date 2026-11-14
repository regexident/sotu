use sotu::prelude::*;
mod borrowing {
    use sotu::prelude::*;
    pub struct Blinker;
    #[automatically_derived]
    impl ::core::default::Default for Blinker {
        #[inline]
        fn default() -> Blinker {
            Blinker {}
        }
    }
    impl Blinker {
        fn on() -> Outcome<State> {
            Super
        }
        fn blinking(counter: &mut u32) -> Outcome<State> {
            *counter += 1;
            Handled
        }
        fn without_initial(counter: &mut u32) -> Outcome<State> {
            if *counter > 0 { Handled } else { Super }
        }
    }
    pub enum State {
        On { counter: u32 },
    }
    impl State {
        pub fn on(counter: u32) -> Self {
            Self::On { counter }
        }
    }
    pub enum Superstate<'sub> {
        Blinking { counter: &'sub mut u32 },
        WithoutInitial { counter: &'sub mut u32 },
    }
    impl Superstate<'_> {
        pub fn blinking() -> State {
            State::on(0)
        }
    }
    impl ::sotu::blocking::IntoStateMachine for Blinker {
        type Event = ();
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            Superstate::blinking()
        }
    }
    impl ::sotu::blocking::State<Blinker> for State {
        fn call_handler<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Blinker,
            _event: &(),
            _context: &mut (),
        ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
            match self {
                State::On { .. } => Blinker::on(),
            }
        }
        fn superstate(&mut self) -> ::core::option::Option<Superstate<'_>> {
            match self {
                State::On { counter } => {
                    ::core::option::Option::Some(Superstate::Blinking { counter })
                }
            }
        }
    }
    impl<'sub> ::sotu::blocking::Superstate<Blinker> for Superstate<'sub> {
        fn call_handler<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Blinker,
            _event: &(),
            _context: &mut (),
        ) -> ::sotu::Outcome<State, ::core::convert::Infallible> {
            match self {
                Superstate::Blinking { counter } => Blinker::blinking(counter),
                Superstate::WithoutInitial { counter } => {
                    Blinker::without_initial(counter)
                }
            }
        }
        fn superstate(&mut self) -> ::core::option::Option<Superstate<'_>> {
            match self {
                Superstate::Blinking { .. } => ::core::option::Option::None,
                Superstate::WithoutInitial { counter } => {
                    ::core::option::Option::Some(Superstate::Blinking {
                        counter: &mut **counter,
                    })
                }
            }
        }
        fn same_state(&self, other: &Superstate<'_>) -> bool {
            match (self, other) {
                (Superstate::Blinking { .. }, Superstate::Blinking { .. }) => true,
                (
                    Superstate::WithoutInitial { .. },
                    Superstate::WithoutInitial { .. },
                ) => true,
                _ => false,
            }
        }
    }
    impl ::sotu::private::Submachine for Blinker {
        type Event = ();
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            Superstate::blinking()
        }
    }
    impl ::sotu::private::BlockingSubmachine for Blinker {}
    impl ::sotu::private::ExitlessSubmachine for Blinker {}
}
mod field_less {
    use sotu::prelude::*;
    pub struct Lamp;
    #[automatically_derived]
    impl ::core::default::Default for Lamp {
        #[inline]
        fn default() -> Lamp {
            Lamp {}
        }
    }
    impl Lamp {
        fn member() -> Outcome<State> {
            Super
        }
        fn group() -> Outcome<State> {
            Handled
        }
    }
    pub enum State {
        Member {},
    }
    impl State {
        pub fn member() -> Self {
            Self::Member {}
        }
    }
    pub enum Superstate {
        Group {},
    }
    impl Superstate {
        pub fn group() -> State {
            State::member()
        }
    }
    impl ::sotu::blocking::IntoStateMachine for Lamp {
        type Event = ();
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = Superstate where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            Superstate::group()
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
                State::Member {} => Lamp::member(),
            }
        }
        fn superstate(&mut self) -> ::core::option::Option<Superstate> {
            match self {
                State::Member {} => ::core::option::Option::Some(Superstate::Group {}),
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
                Superstate::Group {} => Lamp::group(),
            }
        }
        fn superstate(&mut self) -> ::core::option::Option<Superstate> {
            match self {
                Superstate::Group {} => ::core::option::Option::None,
            }
        }
        fn same_state(&self, other: &Superstate) -> bool {
            match (self, other) {
                (Superstate::Group { .. }, Superstate::Group { .. }) => true,
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
            Superstate::group()
        }
    }
    impl ::sotu::private::BlockingSubmachine for Lamp {}
    impl ::sotu::private::ExitlessSubmachine for Lamp {}
}
fn main() {
    let mut blinker = borrowing::Blinker.state_machine();
    let mut lamp = field_less::Lamp.state_machine();
    blinker.step(&mut ());
    lamp.step(&mut ());
}
