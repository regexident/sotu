mod common;

use common::trace;
use sotu::prelude::*;

mod door {
    use sotu::prelude::*;

    use super::trace;

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

    pub struct Open;

    pub struct Close;

    #[derive(Event)]
    pub enum Event {
        Open(Open),
        Close(Close),
    }

    #[state_machine(initial = State::opened())]
    impl Door {
        #[state(superstate = frame)]
        fn opened(event: &Event) -> Outcome<State> {
            match event {
                Event::Close(_) => {
                    trace::record("opened handled close");

                    Transition(State::closed())
                }
                Event::Open(_) => {
                    trace::record("opened deferred open");

                    Super
                }
            }
        }

        #[state(superstate = frame)]
        fn closed(event: &Event) -> Outcome<State> {
            match event {
                Event::Open(_) => {
                    trace::record("closed handled open");

                    Transition(State::opened())
                }
                Event::Close(_) => {
                    trace::record("closed deferred close");

                    Super
                }
            }
        }

        #[superstate]
        fn frame(event: &Event) -> Outcome<State> {
            trace::record(match event {
                Event::Open(_) => "frame deferred open",
                Event::Close(_) => "frame deferred close",
            });

            Super
        }
    }
}

mod washer {
    use sotu::prelude::*;

    use super::{door, trace};

    #[derive(Default)]
    pub struct Washer {
        pub door: Option<door::Door>,
    }

    pub struct Start;

    #[derive(Event)]
    pub enum Event {
        Start(Start),
        Open(door::Open),
        Close(door::Close),
    }

    #[state_machine(initial = State::idle())]
    impl Washer {
        #[state]
        fn idle(event: &Event) -> Outcome<State> {
            match event {
                Event::Start(_) => Transition(Superstate::loading()),
                _ => Handled,
            }
        }

        #[submachine(machine = door::Door, field = door)]
        fn loading(event: &Event) -> Outcome<State> {
            trace::record(match event {
                Event::Start(_) => "loading saw start",
                Event::Open(_) => "loading saw open",
                Event::Close(_) => "loading saw close",
            });

            Handled
        }
    }
}

/// Return an initialized `Washer` in `loading` with the child in `opened`,
/// with an empty trace.
fn loading_washer() -> InitializedStateMachine<washer::Washer> {
    let mut machine = washer::Washer::default()
        .uninitialized_state_machine()
        .init(&mut ());

    machine.handle(washer::Start, &mut ());
    trace::take();

    machine
}

#[test]
fn listed_event_reaches_child_leaf() {
    let mut machine = loading_washer();

    machine.handle(door::Close, &mut ());

    assert_eq!(trace::take(), ["opened handled close"]);
    assert!(matches!(
        machine.state(),
        washer::State::Loading(door::State::Closed {})
    ));

    machine.handle(door::Open, &mut ());

    assert_eq!(trace::take(), ["closed handled open"]);
    assert!(matches!(
        machine.state(),
        washer::State::Loading(door::State::Opened {})
    ));
}
