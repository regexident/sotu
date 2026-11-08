use sotu::prelude::*;
pub struct Finisher;
pub enum Done {
    Finished,
}
impl Finisher {
    fn running() -> Outcome<State, Done> {
        Super
    }
    fn group() -> Outcome<State, Done> {
        Exit(Done::Finished)
    }
}
pub enum State {
    Running {},
}
impl State {
    pub fn running() -> Self {
        Self::Running {}
    }
}
pub enum Superstate {
    Group {},
}
impl ::sotu::blocking::IntoStateMachine for Finisher {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Superstate;
    type Exit = Done;
    fn initial() -> State {
        State::running()
    }
}
impl ::sotu::blocking::State<Finisher> for State {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Finisher,
        _event: &(),
        _context: &mut (),
    ) -> ::sotu::Outcome<Self, Done> {
        match self {
            State::Running {} => Finisher::running(),
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate> {
        match self {
            State::Running {} => ::core::option::Option::Some(Superstate::Group {}),
        }
    }
}
impl ::sotu::blocking::Superstate<Finisher> for Superstate {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Finisher,
        _event: &(),
        _context: &mut (),
    ) -> ::sotu::Outcome<State, Done> {
        match self {
            Superstate::Group {} => Finisher::group(),
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
impl ::sotu::private::Submachine for Finisher {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Superstate;
    type Exit = Done;
    fn initial() -> State {
        State::running()
    }
}
impl ::sotu::private::BlockingSubmachine for Finisher {}
fn main() {
    let mut machine = Finisher.state_machine();
    machine.step(&mut ());
}
