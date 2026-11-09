use sotu::prelude::*;
struct Switch {
    flips: u32,
}
#[automatically_derived]
impl ::core::default::Default for Switch {
    #[inline]
    fn default() -> Switch {
        Switch {
            flips: ::core::default::Default::default(),
        }
    }
}
struct Flip;
struct Noop;
enum Event {
    Flip(Flip),
    Noop(Noop),
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
            other => ::core::result::Result::Err(other),
        }
    }
    fn as_variant_ref(&self) -> ::core::option::Option<&Flip> {
        match self {
            Self::Flip(value) => ::core::option::Option::Some(value),
            _ => ::core::option::Option::None,
        }
    }
}
impl ::core::convert::From<Noop> for Event {
    fn from(value: Noop) -> Self {
        Self::Noop(value)
    }
}
impl ::sotu::event::Variant<Noop> for Event {
    fn from_variant(value: Noop) -> Self {
        Self::Noop(value)
    }
    fn into_variant(self) -> ::core::result::Result<Noop, Self> {
        match self {
            Self::Noop(value) => ::core::result::Result::Ok(value),
            other => ::core::result::Result::Err(other),
        }
    }
    fn as_variant_ref(&self) -> ::core::option::Option<&Noop> {
        match self {
            Self::Noop(value) => ::core::option::Option::Some(value),
            _ => ::core::option::Option::None,
        }
    }
}
impl<SotuParentEvent> ::sotu::event::SubEvent<SotuParentEvent> for Event
where
    SotuParentEvent: ::sotu::event::Variant<Flip>,
    SotuParentEvent: ::sotu::event::Variant<Noop>,
{
    fn accepts(parent: &SotuParentEvent) -> bool {
        <SotuParentEvent as ::sotu::event::Variant<Flip>>::as_variant_ref(parent)
            .is_some()
            || <SotuParentEvent as ::sotu::event::Variant<Noop>>::as_variant_ref(parent)
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
        let parent = match <SotuParentEvent as ::sotu::event::Variant<
            Noop,
        >>::into_variant(parent) {
            ::core::result::Result::Ok(value) => {
                return ::core::result::Result::Ok(Self::Noop(value));
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
            Self::Noop(value) => {
                <SotuParentEvent as ::sotu::event::Variant<Noop>>::from_variant(value)
            }
        }
    }
}
impl Switch {
    fn off(&mut self, event: &Event) -> Outcome<State> {
        match event {
            Event::Flip(_) => {
                self.flips += 1;
                Transition(State::on())
            }
            Event::Noop(_) => Super,
        }
    }
    fn on(event: &Event) -> Outcome<State> {
        match event {
            Event::Flip(_) => Transition(State::off()),
            Event::Noop(_) => Super,
        }
    }
}
pub enum State {
    Off {},
    On {},
}
impl State {
    pub fn off() -> Self {
        Self::Off {}
    }
    pub fn on() -> Self {
        Self::On {}
    }
}
impl ::sotu::blocking::IntoStateMachine for Switch {
    type Event = Event;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = () where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::off()
    }
}
impl ::sotu::blocking::State<Switch> for State {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Switch,
        event: &Event,
        _context: &mut (),
    ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
        match self {
            State::Off {} => Switch::off(shared_storage, event),
            State::On {} => Switch::on(event),
        }
    }
}
impl ::sotu::private::Submachine for Switch {
    type Event = Event;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = () where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::off()
    }
}
impl ::sotu::private::BlockingSubmachine for Switch {}
fn main() {
    let mut machine = Switch::default().state_machine();
    machine.handle(Flip, &mut ());
    machine.handle(Noop, &mut ());
    match (&machine.flips, &1) {
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
