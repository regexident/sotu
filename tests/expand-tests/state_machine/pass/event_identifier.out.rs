use sotu::prelude::*;
pub struct Flip;
pub enum Event {
    Flip(Flip),
}
impl ::core::convert::From<Flip> for Event {
    fn from(value: Flip) -> Self {
        Self::Flip(value)
    }
}
impl ::sotu::event::Variant<Flip> for Event {
    fn from_variant(value: Flip) -> Self {
        Self::Flip(value)
    }
    fn into_variant(self) -> ::core::result::Result<Flip, Self> {
        match self {
            Self::Flip(value) => ::core::result::Result::Ok(value),
        }
    }
    fn as_variant_ref(&self) -> ::core::option::Option<&Flip> {
        match self {
            Self::Flip(value) => ::core::option::Option::Some(value),
        }
    }
}
impl<SotuParentEvent> ::sotu::event::SubEvent<SotuParentEvent> for Event
where
    SotuParentEvent: ::sotu::event::Variant<Flip>,
{
    fn accepts(parent: &SotuParentEvent) -> bool {
        <SotuParentEvent as ::sotu::event::Variant<Flip>>::as_variant_ref(parent)
            .is_some()
    }
    fn project(
        parent: SotuParentEvent,
    ) -> ::core::result::Result<Self, SotuParentEvent> {
        let parent = match <SotuParentEvent as ::sotu::event::Variant<
            Flip,
        >>::into_variant(parent) {
            ::core::result::Result::Ok(value) => {
                return ::core::result::Result::Ok(Self::Flip(value));
            }
            ::core::result::Result::Err(parent) => parent,
        };
        ::core::result::Result::Err(parent)
    }
    fn embed(self) -> SotuParentEvent {
        match self {
            Self::Flip(value) => {
                <SotuParentEvent as ::sotu::event::Variant<Flip>>::from_variant(value)
            }
        }
    }
}
pub struct Switch;
#[automatically_derived]
impl ::core::default::Default for Switch {
    #[inline]
    fn default() -> Switch {
        Switch {}
    }
}
impl Switch {
    fn off(input: &Event) -> Outcome<State> {
        match input {
            Event::Flip(_) => Transition(State::on()),
        }
    }
    fn on(input: &Event, event: &mut u32) -> Outcome<State> {
        *event += 1;
        match input {
            Event::Flip(_) => Transition(State::off()),
        }
    }
}
pub enum State {
    Off {},
    On { event: u32 },
}
impl State {
    pub fn off() -> Self {
        Self::Off {}
    }
    pub fn on() -> Self {
        Self::On {
            event: ::core::default::Default::default(),
        }
    }
}
impl ::sotu::blocking::IntoStateMachine for Switch {
    type Event = Event;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = ();
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::off()
    }
}
impl ::sotu::blocking::State<Switch> for State {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Switch,
        event: &Event,
        _context: &mut (),
    ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
        match self {
            State::Off {} => Switch::off(event),
            State::On { event } => Switch::on(event, event),
        }
    }
}
impl ::sotu::private::Submachine for Switch {
    type Event = Event;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = ();
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::off()
    }
}
impl ::sotu::private::BlockingSubmachine for Switch {}
fn main() {
    let mut machine = Switch.state_machine();
    machine.handle(Flip, &mut ());
}
