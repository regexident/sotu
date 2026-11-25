#![cfg(feature = "async")]

mod common;

use futures::executor::block_on;
use sotu::prelude::*;

// MARK: Door And Washer

mod events {
    use sotu::prelude::*;

    pub struct Start;

    pub struct Open;

    pub struct Close;

    pub struct Tick;

    #[derive(Event)]
    pub enum DoorEvent {
        Open(Open),
        Close(Close),
    }
}

/// The exit points through which both doors leave.
pub enum DoorExit {
    Closed,
}

mod blocking_door {
    use sotu::prelude::*;

    use super::DoorExit;
    use super::events::DoorEvent;

    #[derive(Default)]
    pub struct Door {
        pub openings: u32,
    }

    #[state_machine(initial = State::opened(), exit = DoorExit)]
    impl Door {
        #[state(entry_action = count_opening)]
        fn opened(event: &DoorEvent) -> Outcome<State, DoorExit> {
            match event {
                DoorEvent::Close(_) => Exit(DoorExit::Closed),
                DoorEvent::Open(_) => Handled,
            }
        }

        #[action]
        fn count_opening(&mut self) {
            self.openings += 1;
        }
    }
}

/// Generate an awaitable washer in module `$name` that mounts the door of
/// module `$door` while it loads, so that one scenario runs against a blocking
/// and an awaitable child.
macro_rules! washer {
    ($name:ident, $door:ident) => {
        mod $name {
            use sotu::prelude::*;

            use super::DoorExit;
            use super::events::{Close, Open, Start, Tick};
            use super::$door as door;

            #[derive(Event)]
            pub enum Event {
                Start(Start),
                Tick(Tick),
                Open(Open),
                Close(Close),
            }

            #[derive(Default)]
            pub struct Washer {
                pub cycles: u32,
                pub door: Option<door::Door>,
            }

            #[state_machine(initial = State::idle(), awaitable)]
            impl Washer {
                #[state]
                fn idle(event: &Event) -> Outcome<State> {
                    match event {
                        Event::Start(_) => Transition(Superstate::loading()),
                        _ => Super,
                    }
                }

                #[submachine(machine = door::Door, field = door, exit_handler = door_exited)]
                fn loading(submachine: &mut door::Door, event: &Event) -> Outcome<State> {
                    match event {
                        Event::Tick(_) if submachine.openings > 3 => Transition(State::idle()),
                        _ => Super,
                    }
                }

                #[exit_handler]
                fn door_exited(exit: DoorExit) -> Outcome<State> {
                    match exit {
                        DoorExit::Closed => Transition(State::washing()),
                    }
                }

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
        }
    };
}

washer!(washer_with_blocking_door, blocking_door);

#[test]
fn awaitable_root_mounts_blocking_child() {
    use events::{Close, Start, Tick};
    use washer_with_blocking_door::{State, Washer};

    let mut machine = Washer::default().state_machine();

    block_on(machine.init(&mut ()));
    block_on(machine.handle(Start, &mut ()));

    assert!(matches!(
        machine.state(),
        State::Loading(blocking_door::State::Opened {})
    ));
    assert_eq!(machine.door.as_ref().map(|door| door.openings), Some(1));

    block_on(machine.handle(Tick, &mut ()));

    assert!(matches!(
        machine.state(),
        State::Loading(blocking_door::State::Opened {})
    ));

    block_on(machine.handle(Close, &mut ()));

    assert!(matches!(machine.state(), State::Washing {}));
    assert!(machine.door.is_none());

    block_on(machine.handle(Tick, &mut ()));

    assert!(matches!(machine.state(), State::Idle {}));
    assert_eq!(machine.cycles, 1);
}

mod awaitable_door {
    use sotu::prelude::*;

    use super::DoorExit;
    use super::common::pending_once::pending_once;
    use super::events::DoorEvent;

    #[derive(Default)]
    pub struct Door {
        pub openings: u32,
    }

    #[state_machine(initial = State::opened(), exit = DoorExit)]
    impl Door {
        #[state(entry_action = count_opening)]
        fn opened(event: &DoorEvent) -> Outcome<State, DoorExit> {
            match event {
                DoorEvent::Close(_) => Exit(DoorExit::Closed),
                DoorEvent::Open(_) => Handled,
            }
        }

        #[action]
        async fn count_opening(&mut self) {
            pending_once().await;

            self.openings += 1;
        }
    }
}

washer!(washer_with_awaitable_door, awaitable_door);

#[test]
fn awaitable_root_mounts_awaitable_child() {
    use events::{Close, Start, Tick};
    use washer_with_awaitable_door::{State, Washer};

    let mut machine = Washer::default().state_machine();

    block_on(machine.init(&mut ()));
    block_on(machine.handle(Start, &mut ()));

    assert!(matches!(
        machine.state(),
        State::Loading(awaitable_door::State::Opened {})
    ));
    assert_eq!(machine.door.as_ref().map(|door| door.openings), Some(1));

    block_on(machine.handle(Tick, &mut ()));

    assert!(matches!(
        machine.state(),
        State::Loading(awaitable_door::State::Opened {})
    ));

    block_on(machine.handle(Close, &mut ()));

    assert!(matches!(machine.state(), State::Washing {}));
    assert!(machine.door.is_none());

    block_on(machine.handle(Tick, &mut ()));

    assert!(matches!(machine.state(), State::Idle {}));
    assert_eq!(machine.cycles, 1);
}
