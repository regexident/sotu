use sotu::prelude::*;
pub(crate) struct Switch;
impl Switch {
    fn off() -> Outcome<State> {
        Transition(State::on())
    }
    fn on() -> Outcome<State> {
        Super
    }
    fn wired() -> Outcome<State> {
        Handled
    }
}
pub(crate) enum State {
    Off {},
    On {},
}
impl State {
    pub(crate) fn off() -> Self {
        Self::Off {}
    }
    pub(crate) fn on() -> Self {
        Self::On {}
    }
}
pub(crate) enum Superstate {
    Wired {},
}
impl Superstate {
    pub(crate) fn wired() -> State {
        State::off()
    }
}
impl ::sotu::blocking::IntoStateMachine for Switch {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Superstate;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::off()
    }
}
impl ::sotu::blocking::State<Switch> for State {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Switch,
        _event: &(),
        _context: &mut (),
    ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
        match self {
            State::Off {} => Switch::off(),
            State::On {} => Switch::on(),
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate> {
        match self {
            State::Off {} => ::core::option::Option::Some(Superstate::Wired {}),
            State::On {} => ::core::option::Option::Some(Superstate::Wired {}),
        }
    }
}
impl ::sotu::blocking::Superstate<Switch> for Superstate {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Switch,
        _event: &(),
        _context: &mut (),
    ) -> ::sotu::Outcome<State, ::core::convert::Infallible> {
        match self {
            Superstate::Wired {} => Switch::wired(),
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate> {
        match self {
            Superstate::Wired {} => ::core::option::Option::None,
        }
    }
    fn same_state(&self, other: &Superstate) -> bool {
        match (self, other) {
            (Superstate::Wired { .. }, Superstate::Wired { .. }) => true,
        }
    }
}
impl ::sotu::private::Submachine for Switch {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Superstate;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::off()
    }
}
impl ::sotu::private::BlockingSubmachine for Switch {}
fn main() {
    let mut machine = Switch.state_machine();
    machine.step(&mut ());
}
