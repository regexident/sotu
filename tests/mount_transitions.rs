mod common;

use common::trace;
use sotu::prelude::*;

// MARK: Machines

mod door {
    use sotu::prelude::*;

    use super::trace;

    /// A child whose states both sit under the superstate `frame`, which
    /// answers `Slam` with an external transition to `closed`.
    pub struct Door;

    impl Default for Door {
        fn default() -> Self {
            trace::record("door created");

            Self
        }
    }

    impl Drop for Door {
        fn drop(&mut self) {
            trace::record("door dropped");
        }
    }

    pub struct Slam;

    #[derive(Event)]
    pub enum Event {
        Slam(Slam),
    }

    #[state_machine(initial = State::opened())]
    impl Door {
        #[state(superstate = frame, entry_action = enter_opened, exit_action = exit_opened)]
        fn opened() -> Outcome<State> {
            Super
        }

        #[state(superstate = frame, entry_action = enter_closed, exit_action = exit_closed)]
        fn closed() -> Outcome<State> {
            Super
        }

        #[superstate(entry_action = enter_frame, exit_action = exit_frame)]
        fn frame(event: &Event) -> Outcome<State> {
            match event {
                Event::Slam(_) => ExternalTransition(State::closed()),
            }
        }

        #[action]
        fn enter_opened() {
            trace::record("enter opened");
        }

        #[action]
        fn exit_opened() {
            trace::record("exit opened");
        }

        #[action]
        fn enter_closed() {
            trace::record("enter closed");
        }

        #[action]
        fn exit_closed() {
            trace::record("exit closed");
        }

        #[action]
        fn enter_frame() {
            trace::record("enter frame");
        }

        #[action]
        fn exit_frame() {
            trace::record("exit frame");
        }
    }
}

mod house {
    use sotu::prelude::*;

    use super::{door, trace};

    /// A parent that mounts the door inside the superstate `group`.
    #[derive(Default)]
    pub struct House {
        pub door: Option<door::Door>,
    }

    pub struct Local;

    pub struct External;

    pub struct Regroup;

    #[derive(Event)]
    pub enum Event {
        Local(Local),
        External(External),
        Regroup(Regroup),
        Slam(door::Slam),
    }

    #[state_machine(initial = State::loading(door::State::opened()))]
    impl House {
        #[submachine(machine = door::Door, field = door, superstate = group)]
        fn loading(event: &Event) -> Outcome<State> {
            match event {
                Event::Local(_) => Transition(State::loading(door::State::closed())),
                Event::External(_) => ExternalTransition(State::loading(door::State::closed())),
                Event::Regroup(_) | Event::Slam(_) => Super,
            }
        }

        #[superstate(entry_action = enter_group, exit_action = exit_group)]
        fn group(event: &Event) -> Outcome<State> {
            match event {
                Event::Regroup(_) => ExternalTransition(State::loading(door::State::opened())),
                Event::Local(_) | Event::External(_) | Event::Slam(_) => Handled,
            }
        }

        #[action]
        fn enter_group() {
            trace::record("enter group");
        }

        #[action]
        fn exit_group() {
            trace::record("exit group");
        }
    }
}

/// Return an initialized `House` with the door in `opened`, with an empty
/// trace.
fn opened_house() -> InitializedStateMachine<house::House> {
    let machine = house::House::default()
        .uninitialized_state_machine()
        .init(&mut ());

    assert_eq!(
        trace::take(),
        ["enter group", "door created", "enter frame", "enter opened"]
    );

    machine
}

/// Return how often a door was created in `entries`, counting the creation
/// on initialization that `opened_house` consumed.
fn door_creations(entries: &[&str]) -> usize {
    let later = entries
        .iter()
        .filter(|entry| **entry == "door created")
        .count();

    1 + later
}

// MARK: Tests

#[test]
fn external_transition_from_mount_handler_creates_fresh_instance() {
    let mut machine = opened_house();

    machine.handle(house::External, &mut ());

    let entries = trace::take();

    assert_eq!(
        entries,
        [
            "exit opened",
            "exit frame",
            "door dropped",
            "door created",
            "enter frame",
            "enter closed",
        ]
    );
    assert_eq!(door_creations(&entries), 2);
}

#[test]
fn local_transition_from_mount_handler_keeps_instance() {
    let mut machine = opened_house();

    machine.handle(house::Local, &mut ());

    let entries = trace::take();

    assert_eq!(entries, ["exit opened", "enter closed"]);
    assert_eq!(door_creations(&entries), 1);
    assert!(matches!(
        machine.state(),
        house::State::Loading(door::State::Closed {})
    ));
}

#[test]
fn external_transition_from_enclosing_superstate_reenters_mount() {
    let mut machine = opened_house();

    machine.handle(house::Regroup, &mut ());

    assert_eq!(
        trace::take(),
        [
            "exit opened",
            "exit frame",
            "door dropped",
            "exit group",
            "enter group",
            "door created",
            "enter frame",
            "enter opened",
        ]
    );
    assert!(machine.door.is_some());
}

#[test]
fn child_internal_external_transition_keeps_instance() {
    let mut machine = opened_house();

    machine.handle(door::Slam, &mut ());

    assert_eq!(
        trace::take(),
        ["exit opened", "exit frame", "enter frame", "enter closed"]
    );
    assert!(matches!(
        machine.state(),
        house::State::Loading(door::State::Closed {})
    ));
}
