use sotu::prelude::*;
struct Door {
    openings: u32,
}
#[automatically_derived]
impl ::core::default::Default for Door {
    #[inline]
    fn default() -> Door {
        Door {
            openings: ::core::default::Default::default(),
        }
    }
}
enum Event {
    Toggle,
}
impl Door {
    /// The door is closed.
    ///
    /// Toggling it opens it.
    #[inline]
    fn door_closed(&mut self, event: &Event) -> Outcome<State> {
        self.count_opening();
        match event {
            Event::Toggle => Transition(State::door_open()),
        }
    }
    /// The door is open.
    fn door_open(event: &Event) -> Outcome<State> {
        match event {
            Event::Toggle => Transition(State::door_closed()),
        }
    }
    /// Count one opening of the door.
    fn count_opening(&mut self) {
        self.openings += 1;
    }
}
pub enum State {
    /// The door is closed.
    ///
    /// Toggling it opens it.
    DoorClosed {},
    /// The door is open.
    DoorOpen {},
}
impl State {
    pub fn door_closed() -> Self {
        Self::DoorClosed {}
    }
    pub fn door_open() -> Self {
        Self::DoorOpen {}
    }
}
impl ::sotu::blocking::IntoStateMachine for Door {
    type Event = Event;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = () where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::door_closed()
    }
}
impl ::sotu::blocking::State<Door> for State {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Door,
        event: &Event,
        _context: &mut (),
    ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
        match self {
            State::DoorClosed {} => Door::door_closed(shared_storage, event),
            State::DoorOpen {} => Door::door_open(event),
        }
    }
}
impl ::sotu::private::Submachine for Door {
    type Event = Event;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = () where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::door_closed()
    }
}
impl ::sotu::private::BlockingSubmachine for Door {}
impl ::sotu::private::ExitlessSubmachine for Door {}
fn main() {
    let mut machine = Door::default().state_machine();
    machine.handle(Event::Toggle, &mut ());
    match (&machine.openings, &1) {
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
