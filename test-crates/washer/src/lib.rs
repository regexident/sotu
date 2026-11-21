//! A washer state machine that mounts the door of another crate.
//!
//! This crate is a composition fixture for `sotu`: [`Washer`] mounts
//! [`sotu_test_door::Door`] as a sub-machine while it loads, lists the door's
//! events in its own [`Event`] enum and answers the door's exit with a
//! transition. The code that `#[state_machine]` generates here names the
//! door's items only through their public paths in `sotu_test_door`.
//!
//! The door's [`Tick`](sotu_test_door::Tick) is a shared event type: both
//! [`Event`] and the door's own event enum list it. The washer's hooks record
//! every node in [`Washer::trace`], naming the door's states in the washer's
//! types, for example `"before loading/opened"`.

use sotu::prelude::*;

// MARK: Events

/// The event that starts loading the washer.
pub struct Start;

/// The events that the washer machine handles.
#[derive(Event)]
pub enum Event {
    /// Start loading.
    Start(Start),
    /// Advance the clock, shared with the door.
    Tick(sotu_test_door::Tick),
    /// Open the door.
    Open(sotu_test_door::Open),
    /// Close the door.
    Close(sotu_test_door::Close),
}

// MARK: Machine

/// A washer that mounts a fresh door for every activation of `loading`.
#[derive(Default)]
pub struct Washer {
    /// The number of washing cycles that finished.
    pub cycles: u32,
    /// The entries that the washer's hooks record, oldest first.
    pub trace: Vec<&'static str>,
    door: Option<sotu_test_door::Door>,
}

impl Washer {
    /// Return the mounted door, or `None` while `loading` is inactive.
    pub fn door(&self) -> Option<&sotu_test_door::Door> {
        self.door.as_ref()
    }
}

/// Return the entries that the dispatch hooks record before and after
/// `node`.
fn node_entries(
    node: StateOrSuperstate<'_, State, Superstate<'_>>,
) -> (&'static str, &'static str) {
    match node {
        StateOrSuperstate::State(State::Idle {}) => ("before idle", "after idle"),
        StateOrSuperstate::State(State::Loading(sotu_test_door::State::Opened {})) => {
            ("before loading/opened", "after loading/opened")
        }
        StateOrSuperstate::State(State::Loading(sotu_test_door::State::Ajar {})) => {
            ("before loading/ajar", "after loading/ajar")
        }
        StateOrSuperstate::State(State::Washing {}) => ("before washing", "after washing"),
        StateOrSuperstate::Superstate(Superstate::Loading(SubmachineNode::Mount)) => {
            ("before loading", "after loading")
        }
        // The door declares no superstates, so its superstate type is `()`.
        StateOrSuperstate::Superstate(Superstate::Loading(SubmachineNode::Inner(()))) => {
            ("before loading/superstate", "after loading/superstate")
        }
    }
}

/// Return the entries that the transition hooks record before and after a
/// transition from `source` to `target`.
fn transition_entries(source: &State, target: &State) -> (&'static str, &'static str) {
    use sotu_test_door::State as Door;

    match (source, target) {
        (State::Idle {}, State::Loading(Door::Opened {})) => (
            "transition idle -> loading/opened",
            "transitioned idle -> loading/opened",
        ),
        (State::Loading(Door::Opened {}), State::Idle {}) => (
            "transition loading/opened -> idle",
            "transitioned loading/opened -> idle",
        ),
        (State::Loading(Door::Ajar {}), State::Idle {}) => (
            "transition loading/ajar -> idle",
            "transitioned loading/ajar -> idle",
        ),
        (State::Loading(Door::Opened {}), State::Washing {}) => (
            "transition loading/opened -> washing",
            "transitioned loading/opened -> washing",
        ),
        (State::Loading(Door::Ajar {}), State::Washing {}) => (
            "transition loading/ajar -> washing",
            "transitioned loading/ajar -> washing",
        ),
        (State::Loading(Door::Ajar {}), State::Loading(Door::Opened {})) => (
            "transition loading/ajar -> loading/opened",
            "transitioned loading/ajar -> loading/opened",
        ),
        (State::Washing {}, State::Idle {}) => {
            ("transition washing -> idle", "transitioned washing -> idle")
        }
        // No handler of the washer or the door targets any other state from
        // these sources.
        _ => ("transition other", "transitioned other"),
    }
}

#[state_machine(
    initial = State::idle(),
    before_dispatch = Self::before,
    after_dispatch = Self::after,
    before_transition = Self::enter_transition,
    after_transition = Self::leave_transition
)]
impl Washer {
    fn before(&mut self, node: StateOrSuperstate<'_, State, Superstate<'_>>, _event: &Event) {
        self.trace.push(node_entries(node).0);
    }

    fn after(&mut self, node: StateOrSuperstate<'_, State, Superstate<'_>>, _event: &Event) {
        self.trace.push(node_entries(node).1);
    }

    fn enter_transition(&mut self, source: &State, target: &State) {
        self.trace.push(transition_entries(source, target).0);
    }

    fn leave_transition(&mut self, source: &State, target: &State) {
        self.trace.push(transition_entries(source, target).1);
    }

    /// The washer waits; starting it mounts the door.
    #[state]
    fn idle(event: &Event) -> Outcome<State> {
        match event {
            Event::Start(_) => Transition(Superstate::loading()),
            _ => Super,
        }
    }

    /// The door is mounted while the washer loads; once the door has been
    /// opened more than three times, a tick abandons loading.
    #[submachine(machine = sotu_test_door::Door, field = door, exit_handler = door_exited)]
    fn loading(submachine: &mut sotu_test_door::Door, event: &Event) -> Outcome<State> {
        match event {
            Event::Tick(_) if submachine.openings > 3 => Transition(State::idle()),
            _ => Super,
        }
    }

    #[exit_handler]
    fn door_exited(exit: sotu_test_door::DoorExit) -> Outcome<State> {
        match exit {
            sotu_test_door::DoorExit::Closed => Transition(State::washing()),
        }
    }

    /// The washer washes; a tick finishes the cycle.
    #[state]
    fn washing(&mut self, event: &Event) -> Outcome<State> {
        match event {
            Event::Tick(_) => {
                self.cycles += 1;

                Transition(State::idle())
            }
            _ => Super,
        }
    }
}
