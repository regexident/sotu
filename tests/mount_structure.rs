mod common;

use common::trace;
use sotu::blocking::StateExt;
use sotu::prelude::*;

mod door {
    use sotu::prelude::*;

    use super::trace;

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
        #[state(superstate = frame, entry_action = enter_opened, exit_action = exit_opened)]
        fn opened(event: &Event) -> Outcome<State> {
            match *event {}
        }

        #[state(superstate = frame, entry_action = enter_closed, exit_action = exit_closed)]
        fn closed(event: &Event) -> Outcome<State> {
            match *event {}
        }

        #[superstate(entry_action = enter_frame, exit_action = exit_frame)]
        fn frame(event: &Event) -> Outcome<State> {
            match *event {}
        }

        #[action]
        fn enter_opened(&mut self) {
            self.openings += 1;
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

mod hall {
    use sotu::prelude::*;

    use super::door;

    #[derive(Default)]
    pub struct Hall {
        pub door: Option<door::Door>,
    }

    #[state_machine(initial = State::idle())]
    impl Hall {
        #[state]
        fn idle() -> Outcome<State> {
            Handled
        }

        #[submachine(machine = door::Door, field = door)]
        fn loading() -> Outcome<State> {
            Handled
        }
    }
}

#[test]
fn depth_crosses_mount_boundary() {
    let mut opened = hall::State::loading(door::State::opened());
    let mut closed = hall::State::loading(door::State::closed());
    let mut idle = hall::State::idle();

    assert_eq!(StateExt::<hall::Hall>::depth(&mut opened), 3);
    assert_eq!(StateExt::<hall::Hall>::depth(&mut idle), 1);
    assert!(opened.same_state(&hall::State::loading(door::State::opened())));
    assert!(!opened.same_state(&closed));
    assert!(!opened.same_state(&idle));
    assert_eq!(opened.transition_path(&mut closed), (1, 1));
    assert_eq!(opened.transition_path(&mut idle), (3, 1));
    assert_eq!(idle.transition_path(&mut opened), (1, 3));
}
