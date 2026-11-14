use sotu::prelude::*;
pub enum Event {
    Next,
}
pub struct Linked;
#[automatically_derived]
impl ::core::default::Default for Linked {
    #[inline]
    fn default() -> Linked {
        Linked {}
    }
}
impl Linked {
    fn member(count: &mut u32, event: &Event) -> Outcome<State> {
        *count += 1;
        match event {
            Event::Next => Super,
        }
    }
    fn alone(event: &Event) -> Outcome<State> {
        match event {
            Event::Next => Transition(State::member(0)),
        }
    }
    fn group(count: &u32, event: &Event) -> Outcome<State> {
        match event {
            Event::Next if *count > 1 => Transition(State::alone()),
            Event::Next => Handled,
        }
    }
}
pub enum State {
    Member { count: u32 },
    Alone {},
}
impl State {
    pub fn member(count: u32) -> Self {
        Self::Member { count }
    }
    pub fn alone() -> Self {
        Self::Alone {}
    }
}
pub enum Superstate<'sub> {
    Group { count: &'sub u32 },
}
impl ::sotu::blocking::IntoStateMachine for Linked {
    type Event = Event;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::member(0)
    }
}
impl ::sotu::blocking::State<Linked> for State {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Linked,
        event: &Event,
        _context: &mut (),
    ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
        match self {
            State::Member { count } => Linked::member(count, event),
            State::Alone {} => Linked::alone(event),
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate<'_>> {
        match self {
            State::Member { count } => {
                ::core::option::Option::Some(Superstate::Group { count })
            }
            State::Alone {} => ::core::option::Option::None,
        }
    }
}
impl<'sub> ::sotu::blocking::Superstate<Linked> for Superstate<'sub> {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Linked,
        event: &Event,
        _context: &mut (),
    ) -> ::sotu::Outcome<State, ::core::convert::Infallible> {
        match self {
            Superstate::Group { count } => Linked::group(count, event),
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate<'_>> {
        match self {
            Superstate::Group { .. } => ::core::option::Option::None,
        }
    }
    fn same_state(&self, other: &Superstate<'_>) -> bool {
        match (self, other) {
            (Superstate::Group { .. }, Superstate::Group { .. }) => true,
        }
    }
}
impl ::sotu::private::Submachine for Linked {
    type Event = Event;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::member(0)
    }
}
impl ::sotu::private::BlockingSubmachine for Linked {}
impl ::sotu::private::ExitlessSubmachine for Linked {}
fn main() {
    let mut machine = Linked.state_machine();
    machine.handle(Event::Next, &mut ());
}
