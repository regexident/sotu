use sotu::prelude::*;
pub enum Event {
    Next,
}
pub struct Hierarchy;
#[automatically_derived]
impl ::core::default::Default for Hierarchy {
    #[inline]
    fn default() -> Hierarchy {
        Hierarchy {}
    }
}
impl Hierarchy {
    /// The state below every superstate.
    fn leaf(event: &Event) -> Outcome<State> {
        match event {
            Event::Next => Super,
        }
    }
    fn top(event: &Event) -> Outcome<State> {
        match event {
            Event::Next => Transition(State::leaf()),
        }
    }
    /// The nearest superstate of `leaf`.
    fn inner(event: &Event) -> Outcome<State> {
        match event {
            Event::Next => Super,
        }
    }
    /// The outermost superstate, renamed.
    fn outer(event: &Event) -> Outcome<State> {
        match event {
            Event::Next => Transition(State::top()),
        }
    }
}
pub enum State {
    /// The state below every superstate.
    Leaf {},
    Top {},
}
impl State {
    pub fn leaf() -> Self {
        Self::Leaf {}
    }
    pub fn top() -> Self {
        Self::Top {}
    }
}
pub enum Superstate {
    /// The nearest superstate of `leaf`.
    Inner {},
    /// The outermost superstate, renamed.
    Outermost {},
}
impl ::sotu::blocking::IntoStateMachine for Hierarchy {
    type Event = Event;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Superstate where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::leaf()
    }
}
impl ::sotu::blocking::State<Hierarchy> for State {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Hierarchy,
        event: &Event,
        _context: &mut (),
    ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
        match self {
            State::Leaf {} => Hierarchy::leaf(event),
            State::Top {} => Hierarchy::top(event),
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate> {
        match self {
            State::Leaf {} => ::core::option::Option::Some(Superstate::Inner {}),
            State::Top {} => ::core::option::Option::None,
        }
    }
}
impl ::sotu::blocking::Superstate<Hierarchy> for Superstate {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Hierarchy,
        event: &Event,
        _context: &mut (),
    ) -> ::sotu::Outcome<State, ::core::convert::Infallible> {
        match self {
            Superstate::Inner {} => Hierarchy::inner(event),
            Superstate::Outermost {} => Hierarchy::outer(event),
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate> {
        match self {
            Superstate::Inner {} => {
                ::core::option::Option::Some(Superstate::Outermost {})
            }
            Superstate::Outermost {} => ::core::option::Option::None,
        }
    }
    fn same_state(&self, other: &Superstate) -> bool {
        match (self, other) {
            (Superstate::Inner { .. }, Superstate::Inner { .. }) => true,
            (Superstate::Outermost { .. }, Superstate::Outermost { .. }) => true,
            _ => false,
        }
    }
}
impl ::sotu::private::Submachine for Hierarchy {
    type Event = Event;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Superstate where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::leaf()
    }
}
impl ::sotu::private::BlockingSubmachine for Hierarchy {}
fn main() {
    let mut machine = Hierarchy.state_machine();
    machine.handle(Event::Next, &mut ());
}
