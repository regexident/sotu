use sotu::prelude::*;
pub enum Event {
    Next,
}
pub struct Gauge {
    pub total: u32,
}
#[automatically_derived]
impl ::core::default::Default for Gauge {
    #[inline]
    fn default() -> Gauge {
        Gauge {
            total: ::core::default::Default::default(),
        }
    }
}
impl Gauge {
    /// The leaf holds more fields than its superstates borrow.
    fn measuring(reading: &mut u32, scale: &u32) -> Outcome<State> {
        *reading += *scale;
        Super
    }
    fn stopped(event: &Event) -> Outcome<State> {
        match event {
            Event::Next => Handled,
        }
    }
    /// A mutable borrow of `reading` and a shared borrow of `scale`.
    fn writing(reading: &mut u32, scale: &u32) -> Outcome<State> {
        *reading *= *scale;
        Super
    }
    /// Reborrows `reading` for reading only and does not borrow the other fields.
    fn reading_only(&mut self, reading: &u32) -> Outcome<State> {
        self.total = *reading;
        Super
    }
    /// A superstate at the top that borrows nothing.
    fn stopping(event: &Event) -> Outcome<State> {
        match event {
            Event::Next => Transition(State::stopped()),
        }
    }
}
pub enum State {
    /// The leaf holds more fields than its superstates borrow.
    Measuring { reading: u32, scale: u32, unit: char },
    Stopped {},
}
impl State {
    pub fn measuring(reading: u32, scale: u32, unit: char) -> Self {
        Self::Measuring {
            reading,
            scale,
            unit,
        }
    }
    pub fn stopped() -> Self {
        Self::Stopped {}
    }
}
pub enum Superstate<'sub> {
    /// A mutable borrow of `reading` and a shared borrow of `scale`.
    Writing { reading: &'sub mut u32, scale: &'sub u32, unit: &'sub mut char },
    /// Reborrows `reading` for reading only and does not borrow the other fields.
    ReadingOnly { reading: &'sub u32 },
    /// A superstate at the top that borrows nothing.
    Stopping {},
}
impl ::sotu::blocking::IntoStateMachine for Gauge {
    type Event = Event;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::measuring(0, 1, 'a')
    }
}
impl ::sotu::blocking::State<Gauge> for State {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Gauge,
        event: &Event,
        _context: &mut (),
    ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
        match self {
            State::Measuring { reading, scale, .. } => Gauge::measuring(reading, scale),
            State::Stopped {} => Gauge::stopped(event),
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate<'_>> {
        match self {
            State::Measuring { reading, scale, unit } => {
                ::core::option::Option::Some(Superstate::Writing {
                    reading,
                    scale,
                    unit,
                })
            }
            State::Stopped {} => ::core::option::Option::None,
        }
    }
}
impl<'sub> ::sotu::blocking::Superstate<Gauge> for Superstate<'sub> {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Gauge,
        event: &Event,
        _context: &mut (),
    ) -> ::sotu::Outcome<State, ::core::convert::Infallible> {
        match self {
            Superstate::Writing { reading, scale, .. } => Gauge::writing(reading, scale),
            Superstate::ReadingOnly { reading } => {
                Gauge::reading_only(shared_storage, reading)
            }
            Superstate::Stopping {} => Gauge::stopping(event),
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate<'_>> {
        match self {
            Superstate::Writing { reading, .. } => {
                ::core::option::Option::Some(Superstate::ReadingOnly {
                    reading: &**reading,
                })
            }
            Superstate::ReadingOnly { .. } => {
                ::core::option::Option::Some(Superstate::Stopping {})
            }
            Superstate::Stopping {} => ::core::option::Option::None,
        }
    }
    fn same_state(&self, other: &Superstate<'_>) -> bool {
        match (self, other) {
            (Superstate::Writing { .. }, Superstate::Writing { .. }) => true,
            (Superstate::ReadingOnly { .. }, Superstate::ReadingOnly { .. }) => true,
            (Superstate::Stopping { .. }, Superstate::Stopping { .. }) => true,
            _ => false,
        }
    }
}
impl ::sotu::private::Submachine for Gauge {
    type Event = Event;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::measuring(0, 1, 'a')
    }
}
impl ::sotu::private::BlockingSubmachine for Gauge {}
fn main() {
    let mut machine = Gauge::default().state_machine();
    machine.handle(Event::Next, &mut ());
}
