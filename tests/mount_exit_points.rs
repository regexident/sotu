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

mod latch {
    use sotu::prelude::*;

    use super::trace;

    /// A grandchild that exits on `Release`.
    #[derive(Default)]
    pub struct Latch;

    impl Drop for Latch {
        fn drop(&mut self) {
            trace::record("latch dropped");
        }
    }

    pub struct Release;

    #[derive(Event)]
    pub enum Event {
        Release(Release),
    }

    pub struct Finished;

    #[state_machine(initial = State::held(), exit = Finished)]
    impl Latch {
        #[state(exit_action = exit_held)]
        fn held(event: &Event) -> Outcome<State, Finished> {
            match event {
                Event::Release(_) => Exit(Finished),
            }
        }

        #[action]
        fn exit_held() {
            trace::record("exit held");
        }
    }
}

mod panel {
    use sotu::prelude::*;

    use super::{latch, trace};

    /// A child that mounts the latch and answers its exit with its own exit.
    #[derive(Default)]
    pub struct Panel {
        pub released: bool,
        pub latch: Option<latch::Latch>,
    }

    impl Drop for Panel {
        fn drop(&mut self) {
            trace::record("panel dropped");
        }
    }

    #[derive(Event)]
    pub enum Event {
        Release(latch::Release),
    }

    pub struct Done;

    #[state_machine(initial = Superstate::locking(), exit = Done)]
    impl Panel {
        #[submachine(
            machine = latch::Latch,
            field = latch,
            exit_action = exit_locking,
            exit_handler = latch_exited
        )]
        fn locking(event: &Event) -> Outcome<State, Done> {
            match event {
                Event::Release(_) => Super,
            }
        }

        #[exit_handler]
        fn latch_exited(&mut self, exit: latch::Finished) -> Outcome<State, Done> {
            let latch::Finished = exit;

            self.released = true;

            Exit(Done)
        }

        #[action]
        fn exit_locking() {
            trace::record("exit locking");
        }
    }
}

mod station {
    use sotu::prelude::*;

    use super::{latch, panel, trace};

    /// A root that mounts the panel and answers its exit with a transition,
    /// reading the panel through `submachine`.
    #[derive(Default)]
    pub struct Station {
        pub panel: Option<panel::Panel>,
    }

    pub struct Start;

    #[derive(Event)]
    pub enum Event {
        Start(Start),
        Release(latch::Release),
    }

    #[state_machine(initial = State::idle())]
    impl Station {
        #[state(entry_action = enter_idle)]
        fn idle(event: &Event) -> Outcome<State> {
            match event {
                Event::Start(_) => Transition(Superstate::operating()),
                Event::Release(_) => Handled,
            }
        }

        #[submachine(
            machine = panel::Panel,
            field = panel,
            exit_action = exit_operating,
            exit_handler = panel_exited
        )]
        fn operating() -> Outcome<State> {
            Super
        }

        #[exit_handler]
        fn panel_exited(submachine: &mut panel::Panel, exit: panel::Done) -> Outcome<State> {
            let panel::Done = exit;

            if submachine.released {
                Transition(State::idle())
            } else {
                Handled
            }
        }

        #[action]
        fn enter_idle() {
            trace::record("enter idle");
        }

        #[action]
        fn exit_operating() {
            trace::record("exit operating");
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
fn exit_propagates_through_two_levels() {
    let mut machine = station::Station::default()
        .uninitialized_state_machine()
        .init(&mut ());

    machine.handle(station::Start, &mut ());
    trace::take();

    assert!(matches!(
        machine.state(),
        station::State::Operating(panel::State::Locking(latch::State::Held {}))
    ));

    machine.handle(latch::Release, &mut ());

    assert_eq!(
        trace::take(),
        [
            "exit held",
            "exit locking",
            "latch dropped",
            "exit operating",
            "panel dropped",
            "enter idle",
        ]
    );
    assert!(matches!(machine.state(), station::State::Idle {}));
    assert!(machine.panel.is_none());
    assert!(!machine.is_terminated());
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
