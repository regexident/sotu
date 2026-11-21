//! A door state machine that exists to be mounted by a machine in another
//! crate.
//!
//! This crate is a composition fixture for `sotu`: the `sotu-test-washer`
//! crate mounts [`Door`] as a sub-machine and lists [`Tick`], [`Open`] and
//! [`Close`] in its own event enum. Every item a mounting crate needs is
//! public, so the code that `#[state_machine]` generates in the parent has to
//! resolve the door's items through their public paths.
//!
//! [`Door`] records its dispatch and transition hooks in [`Door::trace`],
//! naming states in the door's own types, which shows what the child observes
//! while it is mounted.

use sotu::prelude::*;

// MARK: Events

/// The event that opens the door.
pub struct Open;

/// The event that closes the door, which leaves the door machine.
pub struct Close;

/// A clock tick, which the door defers to whatever machine mounts it.
pub struct Tick;

/// The events that the door machine handles.
#[derive(Event)]
pub enum Event {
    /// Open the door.
    Open(Open),
    /// Close the door.
    Close(Close),
    /// Advance the clock.
    Tick(Tick),
}

// MARK: Exit

/// The exit points through which the door machine leaves.
pub enum DoorExit {
    /// The door was closed while open.
    Closed,
}

// MARK: Machine

/// A door that counts how often it was opened and records its hooks.
#[derive(Default)]
pub struct Door {
    /// The number of times the door entered `opened`.
    pub openings: u32,
    /// The entries that the door's hooks record, oldest first.
    pub trace: Vec<&'static str>,
}

/// Return the entries that the dispatch hooks record before and after
/// `node`.
fn node_entries(node: StateOrSuperstate<'_, State, ()>) -> (&'static str, &'static str) {
    match node {
        StateOrSuperstate::State(State::Opened {}) => ("before opened", "after opened"),
        StateOrSuperstate::State(State::Ajar {}) => ("before ajar", "after ajar"),
        StateOrSuperstate::Superstate(()) => ("before superstate", "after superstate"),
    }
}

/// Return the entries that the transition hooks record before and after a
/// transition from `source` to `target`.
fn transition_entries(source: &State, target: &State) -> (&'static str, &'static str) {
    match (source, target) {
        (State::Opened {}, State::Opened {}) => (
            "transition opened -> opened",
            "transitioned opened -> opened",
        ),
        (State::Opened {}, State::Ajar {}) => {
            ("transition opened -> ajar", "transitioned opened -> ajar")
        }
        (State::Ajar {}, State::Opened {}) => {
            ("transition ajar -> opened", "transitioned ajar -> opened")
        }
        (State::Ajar {}, State::Ajar {}) => {
            ("transition ajar -> ajar", "transitioned ajar -> ajar")
        }
    }
}

#[state_machine(
    initial = State::opened(),
    exit = DoorExit,
    before_dispatch = Self::before,
    after_dispatch = Self::after,
    before_transition = Self::enter_transition,
    after_transition = Self::leave_transition
)]
impl Door {
    fn before(&mut self, node: StateOrSuperstate<'_, State, ()>, _event: &Event) {
        self.trace.push(node_entries(node).0);
    }

    fn after(&mut self, node: StateOrSuperstate<'_, State, ()>, _event: &Event) {
        self.trace.push(node_entries(node).1);
    }

    fn enter_transition(&mut self, source: &State, target: &State) {
        self.trace.push(transition_entries(source, target).0);
    }

    fn leave_transition(&mut self, source: &State, target: &State) {
        self.trace.push(transition_entries(source, target).1);
    }

    /// The door stands open; closing it leaves the machine, and a tick is
    /// left to the machine that mounts the door.
    #[state(entry_action = count_opening)]
    fn opened(event: &Event) -> Outcome<State, DoorExit> {
        match event {
            Event::Close(_) => Exit(DoorExit::Closed),
            Event::Open(_) => Handled,
            Event::Tick(_) => Super,
        }
    }

    /// The door is ajar; opening it fully enters `opened`.
    #[state]
    fn ajar(event: &Event) -> Outcome<State, DoorExit> {
        match event {
            Event::Open(_) => Transition(State::opened()),
            Event::Close(_) | Event::Tick(_) => Super,
        }
    }

    #[action]
    fn count_opening(&mut self) {
        self.openings += 1;
    }
}
