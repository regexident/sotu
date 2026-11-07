use sotu::prelude::*;
pub struct Probe {
    pub seen: u32,
}
#[automatically_derived]
impl ::core::default::Default for Probe {
    #[inline]
    fn default() -> Probe {
        Probe {
            seen: ::core::default::Default::default(),
        }
    }
}
impl Probe {
    fn measuring(&mut self, first: &mut u32, second: &mut u32) -> Outcome<State> {
        self.seen += *first + *second;
        Transition(State::idle())
    }
    fn idle() -> Outcome<State> {
        Handled
    }
    fn prepare(second: &mut u32, first: &u32) {
        *second += *first;
    }
    fn prepare_nothing() {}
    fn sample(&mut self, second: &u32) {
        self.seen += *second;
    }
}
pub enum State {
    Measuring { first: u32, second: u32 },
    Idle {},
}
impl State {
    pub fn measuring(first: u32, second: u32) -> Self {
        Self::Measuring { first, second }
    }
    pub fn idle() -> Self {
        Self::Idle {}
    }
}
impl ::sotu::blocking::IntoStateMachine for Probe {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = ();
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::measuring(1, 2)
    }
}
impl ::sotu::blocking::State<Probe> for State {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Probe,
        _event: &(),
        _context: &mut (),
    ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
        match self {
            State::Measuring { first, second } => {
                Probe::measuring(shared_storage, first, second)
            }
            State::Idle {} => Probe::idle(),
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Probe,
        _context: &mut (),
    ) {
        match self {
            State::Measuring { second, first } => Probe::prepare(second, first),
            State::Idle {} => Probe::prepare_nothing(),
        }
    }
    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Probe,
        _context: &mut (),
    ) {
        match self {
            State::Measuring { second, .. } => Probe::sample(shared_storage, second),
            State::Idle {} => {}
        }
    }
}
impl ::sotu::private::Submachine for Probe {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = ();
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::measuring(1, 2)
    }
}
impl ::sotu::private::BlockingSubmachine for Probe {}
mod no_receiver {
    use sotu::prelude::*;
    pub struct Silent;
    #[automatically_derived]
    impl ::core::default::Default for Silent {
        #[inline]
        fn default() -> Silent {
            Silent {}
        }
    }
    impl Silent {
        fn only(count: &mut u32) -> Outcome<State> {
            *count += 1;
            Handled
        }
        fn bump(count: &mut u32) {
            *count += 1;
        }
    }
    pub enum State {
        Only { count: u32 },
    }
    impl State {
        pub fn only(count: u32) -> Self {
            Self::Only { count }
        }
    }
    impl ::sotu::blocking::IntoStateMachine for Silent {
        type Event = ();
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = ();
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::only(0)
        }
    }
    impl ::sotu::blocking::State<Silent> for State {
        fn call_handler<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Silent,
            _event: &(),
            _context: &mut (),
        ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
            match self {
                State::Only { count } => Silent::only(count),
            }
        }
        fn call_entry_action<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Silent,
            _context: &mut (),
        ) {
            match self {
                State::Only { count } => Silent::bump(count),
            }
        }
        fn call_exit_action<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Silent,
            _context: &mut (),
        ) {
            match self {
                State::Only { count } => Silent::bump(count),
            }
        }
    }
    impl ::sotu::private::Submachine for Silent {
        type Event = ();
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = ();
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::only(0)
        }
    }
    impl ::sotu::private::BlockingSubmachine for Silent {}
}
fn main() {
    let mut probe = Probe::default().state_machine();
    let mut silent = no_receiver::Silent.state_machine();
    probe.step(&mut ());
    silent.step(&mut ());
    match (&probe.seen, &7) {
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
