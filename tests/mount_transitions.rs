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

mod inner {
    use sotu::prelude::*;

    use super::trace;

    /// A grandchild that exits on `Finish`.
    pub struct Inner;

    impl Default for Inner {
        fn default() -> Self {
            trace::record("inner created");

            Self
        }
    }

    impl Drop for Inner {
        fn drop(&mut self) {
            trace::record("inner dropped");
        }
    }

    pub struct Finish;

    #[derive(Event)]
    pub enum Event {
        Finish(Finish),
    }

    pub struct Finished;

    #[state_machine(initial = State::running(), exit = Finished)]
    impl Inner {
        #[state(exit_action = exit_running)]
        fn running(event: &Event) -> Outcome<State, Finished> {
            match event {
                Event::Finish(_) => Exit(Finished),
            }
        }

        #[action]
        fn exit_running() {
            trace::record("exit running");
        }
    }
}

mod outer {
    use sotu::prelude::*;

    use super::{inner, trace};

    /// A child that mounts the inner machine and answers its exit with its
    /// own exit.
    pub struct Outer {
        pub inner: Option<inner::Inner>,
    }

    impl Default for Outer {
        fn default() -> Self {
            trace::record("outer created");

            Self { inner: None }
        }
    }

    impl Drop for Outer {
        fn drop(&mut self) {
            trace::record("outer dropped");
        }
    }

    #[derive(Event)]
    pub enum Event {
        Finish(inner::Finish),
    }

    pub struct Finished;

    #[state_machine(initial = Superstate::working(), exit = Finished)]
    impl Outer {
        #[submachine(
            machine = inner::Inner,
            field = inner,
            exit_action = exit_working,
            exit_handler = inner_exited
        )]
        fn working(event: &Event) -> Outcome<State, Finished> {
            match event {
                Event::Finish(_) => Super,
            }
        }

        #[exit_handler]
        fn inner_exited(exit: inner::Finished) -> Outcome<State, Finished> {
            let inner::Finished = exit;

            Exit(Finished)
        }

        #[action]
        fn exit_working() {
            trace::record("exit working");
        }
    }
}

mod plant {
    use sotu::prelude::*;

    use super::{inner, outer, trace};

    #[derive(Debug, PartialEq)]
    pub enum RootExit {
        Done,
    }

    /// A root that mounts the outer machine and terminates when it exits.
    #[derive(Default)]
    pub struct Plant {
        pub outer: Option<outer::Outer>,
    }

    #[derive(Event)]
    pub enum Event {
        Finish(inner::Finish),
    }

    #[state_machine(
        initial = Superstate::operating(),
        exit = RootExit,
        before_transition = Self::before_transition,
        after_transition = Self::after_transition
    )]
    impl Plant {
        fn before_transition(&mut self, _source: &State, _target: &State) {
            trace::record("root before transition");
        }

        fn after_transition(&mut self, _source: &State, _target: &State) {
            trace::record("root after transition");
        }

        #[submachine(
            machine = outer::Outer,
            field = outer,
            exit_action = exit_operating,
            exit_handler = outer_exited
        )]
        fn operating(event: &Event) -> Outcome<State, RootExit> {
            match event {
                Event::Finish(_) => Super,
            }
        }

        #[exit_handler]
        fn outer_exited(exit: outer::Finished) -> Outcome<State, RootExit> {
            let outer::Finished = exit;

            Exit(RootExit::Done)
        }

        #[action]
        fn exit_operating() {
            trace::record("exit operating");
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

/// Return an initialized `Plant` that the inner machine has terminated, with
/// an empty trace.
fn terminated_plant() -> InitializedStateMachine<plant::Plant> {
    let mut machine = plant::Plant::default()
        .uninitialized_state_machine()
        .init(&mut ());

    assert_eq!(trace::take(), ["outer created", "inner created"]);
    assert_eq!(
        machine.handle(inner::Finish, &mut ()),
        Some(&plant::RootExit::Done)
    );

    trace::take();

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

#[test]
fn root_termination_from_nested_mount_resets_fields() {
    let mut machine = plant::Plant::default()
        .uninitialized_state_machine()
        .init(&mut ());

    trace::take();

    assert_eq!(
        machine.handle(inner::Finish, &mut ()),
        Some(&plant::RootExit::Done)
    );
    assert_eq!(
        trace::take(),
        [
            "exit running",
            "exit working",
            "inner dropped",
            "exit operating",
            "outer dropped",
        ]
    );
    assert!(machine.outer.is_none());
}

#[test]
fn restart_after_termination_creates_fresh_instances() {
    let mut machine = terminated_plant();

    machine.restart(&mut ());

    assert_eq!(trace::take(), ["outer created", "inner created"]);
    assert!(!machine.is_terminated());
    assert!(matches!(
        machine.state(),
        plant::State::Operating(outer::State::Working(inner::State::Running {}))
    ));
}
