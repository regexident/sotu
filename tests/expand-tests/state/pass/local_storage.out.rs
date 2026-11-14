use sotu::prelude::*;
pub enum Event {
    Next,
}
pub struct Meter {
    pub readings: u32,
}
#[automatically_derived]
impl ::core::default::Default for Meter {
    #[inline]
    fn default() -> Meter {
        Meter {
            readings: ::core::default::Default::default(),
        }
    }
}
impl Meter {
    /// The meter counts.
    fn counting(
        &mut self,
        counter: &mut u32,
        step: &u32,
        event: &Event,
    ) -> Outcome<State> {
        match event {
            Event::Next => {
                *counter += *step;
                self.readings = *counter;
                Transition(State::holding(*counter, 0))
            }
        }
    }
    /// The meter holds a value it does not read while handling events.
    fn holding(event: &Event) -> Outcome<State> {
        match event {
            Event::Next => Transition(State::shadowing(0)),
        }
    }
    /// A field may share its name with a parameter of the generated methods.
    fn shadowing(&mut self, shared_storage: &mut u32) -> Outcome<State> {
        self.readings = *shared_storage;
        Handled
    }
}
pub enum State {
    /// The meter counts.
    Counting { counter: u32, step: u32, started: bool },
    /// The meter holds a value it does not read while handling events.
    Holding { held: u32, label: u8 },
    /// A field may share its name with a parameter of the generated methods.
    Shadowing { shared_storage: u32 },
}
impl State {
    pub fn counting(counter: u32, step: u32, started: bool) -> Self {
        Self::Counting {
            counter,
            step,
            started,
        }
    }
    pub fn holding(held: u32, label: u8) -> Self {
        Self::Holding { held, label }
    }
    pub fn shadowing(shared_storage: u32) -> Self {
        Self::Shadowing { shared_storage }
    }
}
impl ::sotu::blocking::IntoStateMachine for Meter {
    type Event = Event;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = () where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::counting(0, 1, false)
    }
}
impl ::sotu::blocking::State<Meter> for State {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Meter,
        event: &Event,
        _context: &mut (),
    ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
        match self {
            State::Counting { counter, step, .. } => {
                Meter::counting(shared_storage, counter, step, event)
            }
            State::Holding { .. } => Meter::holding(event),
            State::Shadowing { shared_storage } => {
                Meter::shadowing(shared_storage, shared_storage)
            }
        }
    }
}
impl ::sotu::private::Submachine for Meter {
    type Event = Event;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = () where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::counting(0, 1, false)
    }
}
impl ::sotu::private::BlockingSubmachine for Meter {}
impl ::sotu::private::ExitlessSubmachine for Meter {}
fn main() {
    let mut machine = Meter::default().state_machine();
    machine.handle(Event::Next, &mut ());
    match (&machine.readings, &1) {
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
