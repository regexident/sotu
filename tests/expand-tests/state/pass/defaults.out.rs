use sotu::prelude::*;
pub struct Dial;
#[automatically_derived]
impl ::core::default::Default for Dial {
    #[inline]
    fn default() -> Dial {
        Dial {}
    }
}
impl Dial {
    fn all_defaults(lit: &mut bool, level: &u8) -> Outcome<State> {
        *lit = *level > 0;
        Transition(State::mixed(2, 0, 'a'))
    }
    fn mixed(first: &u8, middle: &mut u16, last: &mut u32) -> Outcome<State> {
        *middle += u16::from(*first);
        *last += u32::from(*middle);
        Handled
    }
}
pub enum State {
    AllDefaults { lit: bool, level: u8 },
    Mixed { first: u8, middle: u16, last: u32, tail: char },
}
impl State {
    pub fn all_defaults() -> Self {
        Self::AllDefaults {
            lit: ::core::default::Default::default(),
            level: 3 + 4,
        }
    }
    pub fn mixed(first: u8, last: u32, tail: char) -> Self {
        Self::Mixed {
            first,
            middle: ::core::default::Default::default(),
            last,
            tail,
        }
    }
}
impl ::sotu::blocking::IntoStateMachine for Dial {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = () where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::all_defaults()
    }
}
impl ::sotu::blocking::State<Dial> for State {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Dial,
        _event: &(),
        _context: &mut (),
    ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
        match self {
            State::AllDefaults { lit, level } => Dial::all_defaults(lit, level),
            State::Mixed { first, middle, last, .. } => Dial::mixed(first, middle, last),
        }
    }
}
impl ::sotu::private::Submachine for Dial {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = () where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::all_defaults()
    }
}
impl ::sotu::private::BlockingSubmachine for Dial {}
impl ::sotu::private::ExitlessSubmachine for Dial {}
fn main() {
    let mut machine = Dial.state_machine();
    machine.step(&mut ());
}
