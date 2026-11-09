use sotu::prelude::*;
pub struct Gate;
#[automatically_derived]
impl ::core::default::Default for Gate {
    #[inline]
    fn default() -> Gate {
        Gate {}
    }
}
impl Gate {
    /// The gate stands open.
    fn waiting(ticks: &mut u32) -> Outcome<State> {
        *ticks += 1;
        Transition(State::closed())
    }
    fn shut() -> Outcome<State> {
        Handled
    }
}
pub enum State {
    /// The gate stands open.
    DoorOpen { ticks: u32 },
    Closed {},
}
impl State {
    pub fn door_open(ticks: u32) -> Self {
        Self::DoorOpen { ticks }
    }
    pub fn closed() -> Self {
        Self::Closed {}
    }
}
impl ::sotu::blocking::IntoStateMachine for Gate {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = () where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::door_open(0)
    }
}
impl ::sotu::blocking::State<Gate> for State {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Gate,
        _event: &(),
        _context: &mut (),
    ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
        match self {
            State::DoorOpen { ticks } => Gate::waiting(ticks),
            State::Closed {} => Gate::shut(),
        }
    }
}
impl ::sotu::private::Submachine for Gate {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = () where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::door_open(0)
    }
}
impl ::sotu::private::BlockingSubmachine for Gate {}
fn main() {
    let mut machine = Gate.state_machine();
    machine.step(&mut ());
}
