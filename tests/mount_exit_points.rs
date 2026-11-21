mod common;

use common::trace;
use sotu::prelude::*;

mod door {
    use sotu::prelude::*;

    use super::trace;

    /// A child that exits on `Close` from either of its states, both of which
    /// sit under the superstate `frame`.
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

    pub struct Push;

    pub struct Close;

    #[derive(Event)]
    pub enum Event {
        Push(Push),
        Close(Close),
    }

    pub enum DoorExit {
        Closed,
    }

    #[state_machine(initial = State::opened(), exit = DoorExit)]
    impl Door {
        #[state(superstate = frame, entry_action = enter_opened, exit_action = exit_opened)]
        fn opened(event: &Event) -> Outcome<State, DoorExit> {
            match event {
                Event::Push(_) => Transition(State::closing()),
                Event::Close(_) => Exit(DoorExit::Closed),
            }
        }

        #[state(superstate = frame, entry_action = enter_closing, exit_action = exit_closing)]
        fn closing(event: &Event) -> Outcome<State, DoorExit> {
            match event {
                Event::Push(_) => Super,
                Event::Close(_) => Exit(DoorExit::Closed),
            }
        }

        #[superstate]
        fn frame(event: &Event) -> Outcome<State, DoorExit> {
            trace::record(match event {
                Event::Push(_) => "frame handled push",
                Event::Close(_) => "frame handled close",
            });

            Handled
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
        fn enter_closing() {
            trace::record("enter closing");
        }

        #[action]
        fn exit_closing() {
            trace::record("exit closing");
        }
    }
}

mod laundry {
    use sotu::prelude::*;

    use super::{door, trace};

    /// The outcome with which the exit handler answers the door's exit.
    pub enum Answer {
        Handle,
        Leave,
        Reenter,
        Decline,
    }

    /// A parent whose exit handler answers the door's exit as `answer` says.
    pub struct Laundry {
        pub answer: Answer,
        pub door: Option<door::Door>,
    }

    impl Laundry {
        pub fn new(answer: Answer) -> Self {
            Self { answer, door: None }
        }
    }

    pub struct Start;

    #[derive(Event)]
    pub enum Event {
        Start(Start),
        Push(door::Push),
        Close(door::Close),
    }

    #[state_machine(initial = State::idle())]
    impl Laundry {
        #[state]
        fn idle(event: &Event) -> Outcome<State> {
            match event {
                Event::Start(_) => Transition(Superstate::loading()),
                _ => Super,
            }
        }

        #[submachine(machine = door::Door, field = door, exit_handler = door_exited)]
        fn loading(event: &Event) -> Outcome<State> {
            trace::record(match event {
                Event::Start(_) => "loading handled start",
                Event::Push(_) => "loading handled push",
                Event::Close(_) => "loading handled close",
            });

            Handled
        }

        #[exit_handler]
        fn door_exited(&self, exit: door::DoorExit) -> Outcome<State> {
            match exit {
                door::DoorExit::Closed => match self.answer {
                    Answer::Handle => Handled,
                    Answer::Leave => Transition(State::washing()),
                    Answer::Reenter => ExternalTransition(State::loading(door::State::opened())),
                    Answer::Decline => Super,
                },
            }
        }

        #[state(entry_action = enter_washing)]
        fn washing() -> Outcome<State> {
            Super
        }

        #[action]
        fn enter_washing() {
            trace::record("enter washing");
        }
    }
}

/// Return an initialized `Laundry` that answers the door's exit as `answer`
/// says, in `loading` with the door in `opened`, with an empty trace.
fn loading_laundry(answer: laundry::Answer) -> InitializedStateMachine<laundry::Laundry> {
    let mut machine = laundry::Laundry::new(answer)
        .uninitialized_state_machine()
        .init(&mut ());

    machine.handle(laundry::Start, &mut ());

    assert_eq!(trace::take(), ["door created", "enter opened"]);

    machine
}

#[test]
fn handled_exit_keeps_child_state() {
    let mut machine = loading_laundry(laundry::Answer::Handle);

    machine.handle(door::Push, &mut ());

    assert_eq!(trace::take(), ["exit opened", "enter closing"]);

    machine.handle(door::Close, &mut ());

    assert_eq!(trace::take(), Vec::<&str>::new());
    assert!(matches!(
        machine.state(),
        laundry::State::Loading(door::State::Closing {})
    ));
    assert!(machine.door.is_some());
}

#[test]
fn transition_from_exit_handler_leaves_mount() {
    let mut machine = loading_laundry(laundry::Answer::Leave);

    machine.handle(door::Close, &mut ());

    assert_eq!(
        trace::take(),
        ["exit opened", "door dropped", "enter washing"]
    );
    assert!(matches!(machine.state(), laundry::State::Washing {}));
    assert!(machine.door.is_none());
}

#[test]
fn external_transition_from_exit_handler_reenters_mount() {
    let mut machine = loading_laundry(laundry::Answer::Reenter);

    machine.handle(door::Close, &mut ());

    assert_eq!(
        trace::take(),
        [
            "exit opened",
            "door dropped",
            "door created",
            "enter opened"
        ]
    );
    assert!(matches!(
        machine.state(),
        laundry::State::Loading(door::State::Opened {})
    ));
    assert!(machine.door.is_some());
}

#[test]
fn super_from_exit_handler_continues_at_mount_handler() {
    let mut machine = loading_laundry(laundry::Answer::Decline);

    machine.handle(door::Close, &mut ());

    assert_eq!(trace::take(), ["loading handled close"]);
    assert!(matches!(
        machine.state(),
        laundry::State::Loading(door::State::Opened {})
    ));
}
