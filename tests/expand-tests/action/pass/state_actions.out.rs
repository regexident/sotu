use sotu::prelude::*;
pub enum Event {
    Flip,
}
mod entry_only {
    use sotu::prelude::*;
    use super::Event;
    pub struct Lamp {
        pub flips: u32,
    }
    #[automatically_derived]
    impl ::core::default::Default for Lamp {
        #[inline]
        fn default() -> Lamp {
            Lamp {
                flips: ::core::default::Default::default(),
            }
        }
    }
    impl Lamp {
        fn on(
            &mut self,
            counter: &mut u32,
            lit: &mut bool,
            event: &Event,
        ) -> Outcome<State> {
            match event {
                Event::Flip if *lit => {
                    *counter += 1;
                    self.flips = *counter;
                    Transition(State::off())
                }
                Event::Flip => Handled,
            }
        }
        fn off(event: &Event) -> Outcome<State> {
            match event {
                Event::Flip => Transition(State::on(0, false)),
            }
        }
        fn light(&mut self, lit: &mut bool) {
            self.flips += 1;
            *lit = true;
        }
    }
    pub enum State {
        On { counter: u32, lit: bool, started: bool },
        Off {},
    }
    impl State {
        pub fn on(counter: u32, started: bool) -> Self {
            Self::On {
                counter,
                lit: ::core::default::Default::default(),
                started,
            }
        }
        pub fn off() -> Self {
            Self::Off {}
        }
    }
    impl ::sotu::blocking::IntoStateMachine for Lamp {
        type Event = Event;
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = () where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::on(0, false)
        }
    }
    impl ::sotu::blocking::State<Lamp> for State {
        fn call_handler<'ctx, 'data: 'ctx>(
            &mut self,
            shared_storage: &mut Lamp,
            event: &Event,
            _context: &mut (),
        ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
            match self {
                State::On { counter, lit, .. } => {
                    Lamp::on(shared_storage, counter, lit, event)
                }
                State::Off {} => Lamp::off(event),
            }
        }
        fn call_entry_action<'ctx, 'data: 'ctx>(
            &mut self,
            shared_storage: &mut Lamp,
            _context: &mut (),
        ) {
            match self {
                State::On { lit, .. } => Lamp::light(shared_storage, lit),
                State::Off {} => {}
            }
        }
    }
    impl ::sotu::private::Submachine for Lamp {
        type Event = Event;
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = () where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::on(0, false)
        }
    }
    impl ::sotu::private::BlockingSubmachine for Lamp {}
    impl ::sotu::private::ExitlessSubmachine for Lamp {}
}
mod exit_only {
    use sotu::prelude::*;
    use super::Event;
    pub struct Latch {
        pub total: u32,
    }
    #[automatically_derived]
    impl ::core::default::Default for Latch {
        #[inline]
        fn default() -> Latch {
            Latch {
                total: ::core::default::Default::default(),
            }
        }
    }
    impl Latch {
        fn first(amount: &mut u32, event: &Event) -> Outcome<State> {
            match event {
                Event::Flip => Transition(State::second(*amount + 1)),
            }
        }
        fn second(amount: &mut u32, extra: &mut u8, event: &Event) -> Outcome<State> {
            *extra += 1;
            match event {
                Event::Flip => Transition(State::first(*amount)),
            }
        }
        fn record(&mut self, amount: &mut u32) {
            self.total += *amount;
        }
    }
    pub enum State {
        First { amount: u32 },
        Second { amount: u32, extra: u8 },
    }
    impl State {
        pub fn first(amount: u32) -> Self {
            Self::First { amount }
        }
        pub fn second(amount: u32) -> Self {
            Self::Second {
                amount,
                extra: ::core::default::Default::default(),
            }
        }
    }
    impl ::sotu::blocking::IntoStateMachine for Latch {
        type Event = Event;
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = () where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::first(1)
        }
    }
    impl ::sotu::blocking::State<Latch> for State {
        fn call_handler<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Latch,
            event: &Event,
            _context: &mut (),
        ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
            match self {
                State::First { amount } => Latch::first(amount, event),
                State::Second { amount, extra } => Latch::second(amount, extra, event),
            }
        }
        fn call_exit_action<'ctx, 'data: 'ctx>(
            &mut self,
            shared_storage: &mut Latch,
            _context: &mut (),
        ) {
            match self {
                State::First { amount } => Latch::record(shared_storage, amount),
                State::Second { amount, .. } => Latch::record(shared_storage, amount),
            }
        }
    }
    impl ::sotu::private::Submachine for Latch {
        type Event = Event;
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = () where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::first(1)
        }
    }
    impl ::sotu::private::BlockingSubmachine for Latch {}
    impl ::sotu::private::ExitlessSubmachine for Latch {}
}
fn main() {
    let mut lamp = entry_only::Lamp::default().state_machine();
    let mut latch = exit_only::Latch::default().state_machine();
    lamp.handle(Event::Flip, &mut ());
    latch.handle(Event::Flip, &mut ());
    match (&lamp.flips, &1) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    match (&latch.total, &1) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
}
