mod common;

use common::trace;
use sotu::prelude::*;

mod door {
    use sotu::prelude::*;

    use super::trace;

    /// A child whose creation, drop and state actions are recorded.
    pub struct Door {
        pub openings: u32,
    }

    impl Default for Door {
        fn default() -> Self {
            trace::record("door created");

            Self { openings: 0 }
        }
    }

    impl Drop for Door {
        fn drop(&mut self) {
            trace::record("door dropped");
        }
    }

    #[derive(Event)]
    pub enum Event {}

    #[state_machine(initial = State::opened())]
    impl Door {
        #[state(entry_action = enter_opened, exit_action = exit_opened)]
        fn opened(event: &Event) -> Outcome<State> {
            match *event {}
        }

        #[action]
        fn enter_opened() {
            trace::record("enter opened");
        }

        #[action]
        fn exit_opened() {
            trace::record("exit opened");
        }
    }
}

mod house {
    use sotu::prelude::*;

    use super::{door, trace};

    /// A parent whose mount actions borrow the instance and the shared storage.
    #[derive(Default)]
    pub struct House {
        pub departures: u32,
        pub door: Option<door::Door>,
    }

    pub enum Event {
        Enter,
        Leave,
    }

    #[state_machine(initial = State::idle())]
    impl House {
        #[state]
        fn idle(event: &Event) -> Outcome<State> {
            match event {
                Event::Enter => Transition(Superstate::loading()),
                Event::Leave => Handled,
            }
        }

        #[submachine(machine = door::Door, field = door, entry_action = greet, exit_action = part)]
        fn loading(event: &Event) -> Outcome<State> {
            match event {
                Event::Enter => Handled,
                Event::Leave => Transition(State::idle()),
            }
        }

        #[action]
        fn greet(submachine: &mut door::Door) {
            trace::record("mount entered");

            submachine.openings = 9;
        }

        #[action]
        fn part(&mut self) {
            trace::record("mount exited");

            self.departures += 1;
        }
    }
}

/// Return an initialized `House` in `idle`, with an empty trace.
fn idle_house() -> InitializedStateMachine<house::House> {
    let machine = house::House::default()
        .uninitialized_state_machine()
        .init(&mut ());

    trace::take();

    machine
}

#[test]
fn mount_entry_action_runs_between_creation_and_child_entries() {
    let mut machine = idle_house();

    machine.handle(house::Event::Enter, &mut ());

    assert_eq!(
        trace::take(),
        ["door created", "mount entered", "enter opened"]
    );
}

#[test]
fn mount_exit_action_runs_between_child_exits_and_drop() {
    let mut machine = idle_house();

    machine.handle(house::Event::Enter, &mut ());
    trace::take();

    machine.handle(house::Event::Leave, &mut ());

    assert_eq!(
        trace::take(),
        ["exit opened", "mount exited", "door dropped"]
    );
    assert_eq!(machine.departures, 1);
    assert!(machine.door.is_none());
}

#[test]
fn mount_action_sees_instance() {
    let mut machine = idle_house();

    machine.handle(house::Event::Enter, &mut ());

    assert_eq!(machine.door.as_ref().map(|door| door.openings), Some(9));
}
