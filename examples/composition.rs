//! A washer that mounts a door machine while it loads.
//!
//! `door` is a complete state machine of its own. It counts how often it was
//! opened, and it leaves through the exit point `DoorExit::Closed` when the
//! `Close` event arrives. `washer` mounts that machine as the submachine
//! `loading`:
//!
//! - `idle` enters `loading` on `Start`, which creates a fresh door in its
//!   initial state `opened`.
//! - While `loading` is active, the washer forwards its events to the door. The
//!   door handles `Close` itself, and defers `Tick` to the washer, which is why
//!   both machines list `Tick` in their event enums.
//! - The `exit_handler` `door_exited` maps the door's exit point to a
//!   transition of the washer, here to `washing`. Leaving `loading` releases
//!   the door.
//! - `washing` counts a finished cycle on `Tick` and returns to `idle`.
//!
//! The washer's state type embeds the door's: while the door is mounted, the
//! washer is in `Loading(Opened)`.
//!
//! Run it with `cargo run --example composition`. The output is:
//!
//! ```plain
//! start: Loading(Opened)
//! openings: 1
//! close: Washing
//! door present: false
//! tick: Idle
//! cycles: 1
//! ```

mod door {
    use sotu::prelude::*;

    pub struct Close;

    pub struct Tick;

    #[derive(Event)]
    pub enum Event {
        Close(Close),
        Tick(Tick),
    }

    /// The exit points through which the door machine leaves.
    pub enum DoorExit {
        Closed,
    }

    #[derive(Default)]
    pub struct Door {
        pub openings: u32,
    }

    #[state_machine(initial = State::opened(), exit = DoorExit, state(derive(Debug)))]
    impl Door {
        #[state(entry_action = count_opening)]
        fn opened(event: &Event) -> Outcome<State, DoorExit> {
            match event {
                Event::Close(_) => Exit(DoorExit::Closed),
                Event::Tick(_) => Super,
            }
        }

        #[action]
        fn count_opening(&mut self) {
            self.openings += 1;
        }
    }
}

mod washer {
    use sotu::prelude::*;

    use super::door;

    pub struct Start;

    #[derive(Event)]
    pub enum Event {
        Start(Start),
        Close(door::Close),
        Tick(door::Tick),
    }

    #[derive(Default)]
    pub struct Washer {
        pub cycles: u32,
        door: Option<door::Door>,
    }

    impl Washer {
        /// Return the mounted door, or `None` while `loading` is inactive.
        pub fn door(&self) -> Option<&door::Door> {
            self.door.as_ref()
        }
    }

    #[state_machine(initial = State::idle(), state(derive(Debug)))]
    impl Washer {
        #[state]
        fn idle(event: &Event) -> Outcome<State> {
            match event {
                Event::Start(_) => Transition(Superstate::loading()),
                Event::Close(_) | Event::Tick(_) => Super,
            }
        }

        #[submachine(machine = door::Door, field = door, exit_handler = door_exited)]
        fn loading(event: &Event) -> Outcome<State> {
            match event {
                Event::Start(_) | Event::Close(_) | Event::Tick(_) => Super,
            }
        }

        #[exit_handler]
        fn door_exited(exit: door::DoorExit) -> Outcome<State> {
            match exit {
                door::DoorExit::Closed => Transition(State::washing()),
            }
        }

        #[state]
        fn washing(&mut self, event: &Event) -> Outcome<State> {
            match event {
                Event::Tick(_) => {
                    self.cycles += 1;

                    Transition(State::idle())
                }
                Event::Start(_) | Event::Close(_) => Super,
            }
        }
    }
}

use sotu::prelude::*;
use washer::{Start, Washer};

fn main() {
    let mut machine = Washer::default().state_machine();

    machine.handle(Start, &mut ());
    println!("start: {:?}", machine.state());

    if let Some(door) = machine.door() {
        println!("openings: {}", door.openings);
    }

    machine.handle(door::Close, &mut ());
    println!("close: {:?}", machine.state());
    println!("door present: {}", machine.door().is_some());

    machine.handle(door::Tick, &mut ());
    println!("tick: {:?}", machine.state());
    println!("cycles: {}", machine.cycles);
}
