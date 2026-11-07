use sotu::prelude::*;
struct Ticker {
    ticks: u32,
}
#[automatically_derived]
impl ::core::default::Default for Ticker {
    #[inline]
    fn default() -> Ticker {
        Ticker {
            ticks: ::core::default::Default::default(),
        }
    }
}
impl Ticker {
    fn ticking(&mut self) -> Outcome<State> {
        self.ticks += 1;
        Handled
    }
}
pub enum State {
    Ticking {},
}
impl State {
    pub fn ticking() -> Self {
        Self::Ticking {}
    }
}
impl ::sotu::blocking::IntoStateMachine for Ticker {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = ();
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::ticking()
    }
}
impl ::sotu::blocking::State<Ticker> for State {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Ticker,
        _event: &(),
        _context: &mut (),
    ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
        match self {
            State::Ticking {} => Ticker::ticking(shared_storage),
        }
    }
}
impl ::sotu::private::Submachine for Ticker {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = ();
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::ticking()
    }
}
impl ::sotu::private::BlockingSubmachine for Ticker {}
fn main() {
    let mut machine = Ticker::default().state_machine();
    machine.step(&mut ());
    match (&machine.ticks, &1) {
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
