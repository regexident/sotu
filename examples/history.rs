//! A dishwasher that resumes what it was doing when its door closes.
//!
//! `idle` and `washing` are leaves of the superstate `door_closed`, which
//! handles `Opened` for both by moving to `door_open`. The machine has to know
//! which of the two to return to, so the shared storage keeps a `Resume`
//! value, and the `after_transition` hook `remember` updates it whenever the
//! machine arrives in `idle` or `washing`. Closing the door then leaves
//! `door_open` for the remembered state.
//!
//! Run it with `cargo run --example history`. Each line is the state after one
//! event, in the order `Start`, `Opened`, `Closed`, `Elapsed`, `Opened`,
//! `Closed`:
//!
//! ```plain
//! Washing
//! DoorOpen
//! Washing
//! Idle
//! DoorOpen
//! Idle
//! ```

use sotu::prelude::*;

#[derive(Default)]
enum Resume {
    #[default]
    Idle,
    Washing,
}

#[derive(Default)]
struct Dishwasher {
    resume: Resume,
}

struct Start;

struct Opened;

struct Closed;

struct Elapsed;

#[derive(Event)]
enum Event {
    Start(Start),
    Opened(Opened),
    Closed(Closed),
    Elapsed(Elapsed),
}

#[state_machine(
    initial = State::idle(),
    after_transition = Self::remember,
    state(derive(Debug))
)]
impl Dishwasher {
    #[superstate]
    fn door_closed(event: &Event) -> Outcome<State> {
        match event {
            Event::Opened(_) => Transition(State::door_open()),
            Event::Start(_) | Event::Closed(_) | Event::Elapsed(_) => Super,
        }
    }

    #[state(superstate = door_closed)]
    fn idle(event: &Event) -> Outcome<State> {
        match event {
            Event::Start(_) => Transition(State::washing()),
            Event::Opened(_) | Event::Closed(_) | Event::Elapsed(_) => Super,
        }
    }

    #[state(superstate = door_closed)]
    fn washing(event: &Event) -> Outcome<State> {
        match event {
            Event::Elapsed(_) => Transition(State::idle()),
            Event::Start(_) | Event::Opened(_) | Event::Closed(_) => Super,
        }
    }

    #[state]
    fn door_open(&mut self, event: &Event) -> Outcome<State> {
        match (event, &self.resume) {
            (Event::Closed(_), Resume::Idle) => Transition(State::idle()),
            (Event::Closed(_), Resume::Washing) => Transition(State::washing()),
            (Event::Start(_) | Event::Opened(_) | Event::Elapsed(_), _) => Super,
        }
    }

    /// Record the closed-door state that the machine arrived in.
    ///
    /// Arriving in `door_open` leaves the record alone, because it holds the
    /// state to return to.
    fn remember(&mut self, _source: &State, target: &State) {
        match target {
            State::Idle {} => self.resume = Resume::Idle,
            State::Washing {} => self.resume = Resume::Washing,
            State::DoorOpen {} => {}
        }
    }
}

fn main() {
    let mut machine = Dishwasher::default().state_machine();

    machine.handle(Start, &mut ());
    println!("{:?}", machine.state());

    machine.handle(Opened, &mut ());
    println!("{:?}", machine.state());

    machine.handle(Closed, &mut ());
    println!("{:?}", machine.state());

    machine.handle(Elapsed, &mut ());
    println!("{:?}", machine.state());

    machine.handle(Opened, &mut ());
    println!("{:?}", machine.state());

    machine.handle(Closed, &mut ());
    println!("{:?}", machine.state());
}
