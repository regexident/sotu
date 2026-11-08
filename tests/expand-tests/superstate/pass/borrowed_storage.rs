use sotu::prelude::*;

pub enum Event {
    Next,
}

#[derive(Default)]
pub struct Gauge {
    pub total: u32,
}

#[state_machine(initial = State::measuring(0, 1, 'a'))]
impl Gauge {
    /// The leaf holds more fields than its superstates borrow.
    #[state(superstate = writing, local_storage(unit: char))]
    fn measuring(reading: &mut u32, scale: &u32) -> Outcome<State> {
        *reading += *scale;

        Super
    }

    #[state]
    fn stopped(event: &Event) -> Outcome<State> {
        match event {
            Event::Next => Handled,
        }
    }

    /// A mutable borrow of `reading` and a shared borrow of `scale`.
    #[superstate(superstate = reading_only, local_storage(unit: &'sub mut char))]
    fn writing(reading: &mut u32, scale: &u32) -> Outcome<State> {
        *reading *= *scale;

        Super
    }

    /// Reborrows `reading` for reading only and does not borrow the other fields.
    #[superstate(superstate = stopping)]
    fn reading_only(&mut self, reading: &u32) -> Outcome<State> {
        self.total = *reading;

        Super
    }

    /// A superstate at the top that borrows nothing.
    #[superstate]
    fn stopping(event: &Event) -> Outcome<State> {
        match event {
            Event::Next => Transition(State::stopped()),
        }
    }
}

fn main() {
    let mut machine = Gauge::default().state_machine();

    machine.handle(Event::Next, &mut ());
}
