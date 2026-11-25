#![cfg(feature = "async")]

mod common;

use common::trace;
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

// MARK: Hooks

mod leaf {
    use sotu::prelude::*;

    use super::trace;

    #[derive(Default)]
    pub struct Leaf;

    pub struct Ping;

    pub struct Flip;

    #[derive(Event)]
    pub enum Event {
        Ping(Ping),
        Flip(Flip),
    }

    #[state_machine(
        initial = State::idle(),
        before_dispatch = Self::before,
        after_dispatch = Self::after,
        before_transition = Self::before_transition,
        after_transition = Self::after_transition
    )]
    impl Leaf {
        fn before(&mut self, node: StateOrSuperstate<'_, State, ()>, _event: &Event) {
            trace::record(match node {
                StateOrSuperstate::State(_) => "leaf before leaf-leaf",
                StateOrSuperstate::Superstate(()) => "leaf before superstate",
            });
        }

        fn after(&mut self, node: StateOrSuperstate<'_, State, ()>, _event: &Event) {
            trace::record(match node {
                StateOrSuperstate::State(_) => "leaf after leaf-leaf",
                StateOrSuperstate::Superstate(()) => "leaf after superstate",
            });
        }

        fn before_transition(&mut self, _source: &State, _target: &State) {
            trace::record("leaf before transition");
        }

        fn after_transition(&mut self, _source: &State, _target: &State) {
            trace::record("leaf after transition");
        }

        #[state(entry_action = enter_idle, exit_action = exit_idle)]
        fn idle(event: &Event) -> Outcome<State> {
            match event {
                Event::Ping(_) => {
                    trace::record("leaf-leaf handled");

                    Handled
                }
                Event::Flip(_) => Transition(State::active()),
            }
        }

        #[state(entry_action = enter_active)]
        fn active() -> Outcome<State> {
            Handled
        }

        #[action]
        fn enter_idle() {
            trace::record("idle entered");
        }

        #[action]
        fn exit_idle() {
            trace::record("idle exited");
        }

        #[action]
        fn enter_active() {
            trace::record("active entered");
        }
    }
}

mod middle {
    use sotu::prelude::*;

    use super::common::pending_once::pending_once;
    use super::{leaf, trace};

    #[derive(Default)]
    pub struct Middle {
        pub leaf: Option<leaf::Leaf>,
    }

    pub struct Enter;

    #[derive(Event)]
    pub enum Event {
        Enter(Enter),
        Ping(leaf::Ping),
        Flip(leaf::Flip),
    }

    #[state_machine(
        initial = State::busy(),
        before_dispatch = Self::before,
        after_dispatch = Self::after,
        before_transition = Self::before_transition,
        after_transition = Self::after_transition,
        awaitable
    )]
    impl Middle {
        async fn before(
            &mut self,
            node: StateOrSuperstate<'_, State, Superstate<'_>>,
            _event: &Event,
        ) {
            let entry = match node {
                StateOrSuperstate::State(State::Nesting(_)) => "middle before leaf-leaf",
                _ => "middle before other",
            };

            pending_once().await;

            trace::record(entry);
        }

        async fn after(
            &mut self,
            node: StateOrSuperstate<'_, State, Superstate<'_>>,
            _event: &Event,
        ) {
            let entry = match node {
                StateOrSuperstate::State(State::Nesting(_)) => "middle after leaf-leaf",
                _ => "middle after other",
            };

            pending_once().await;

            trace::record(entry);
        }

        async fn before_transition(&mut self, _source: &State, _target: &State) {
            pending_once().await;

            trace::record("middle before transition");
        }

        async fn after_transition(&mut self, _source: &State, _target: &State) {
            pending_once().await;

            trace::record("middle after transition");
        }

        #[state]
        fn busy(event: &Event) -> Outcome<State> {
            match event {
                Event::Enter(_) => Transition(Superstate::nesting()),
                Event::Ping(_) | Event::Flip(_) => Super,
            }
        }

        #[submachine(machine = leaf::Leaf, field = leaf)]
        fn nesting(event: &Event) -> Outcome<State> {
            match event {
                Event::Enter(_) | Event::Ping(_) | Event::Flip(_) => Handled,
            }
        }
    }
}

mod root {
    use sotu::prelude::*;

    use super::{leaf, middle, trace};

    #[derive(Default)]
    pub struct Root {
        pub middle: Option<middle::Middle>,
    }

    #[derive(Event)]
    pub enum Event {
        Enter(middle::Enter),
        Ping(leaf::Ping),
        Flip(leaf::Flip),
    }

    #[state_machine(
        initial = Superstate::mount(),
        before_dispatch = Self::before,
        after_dispatch = Self::after,
        before_transition = Self::before_transition,
        after_transition = Self::after_transition,
        awaitable
    )]
    impl Root {
        fn before(&mut self, node: StateOrSuperstate<'_, State, Superstate<'_>>, _event: &Event) {
            trace::record(match node {
                StateOrSuperstate::State(State::Mount(middle::State::Nesting(_))) => {
                    "root before leaf-leaf"
                }
                _ => "root before other",
            });
        }

        fn after(&mut self, node: StateOrSuperstate<'_, State, Superstate<'_>>, _event: &Event) {
            trace::record(match node {
                StateOrSuperstate::State(State::Mount(middle::State::Nesting(_))) => {
                    "root after leaf-leaf"
                }
                _ => "root after other",
            });
        }

        fn before_transition(&mut self, _source: &State, _target: &State) {
            trace::record("root before transition");
        }

        fn after_transition(&mut self, _source: &State, _target: &State) {
            trace::record("root after transition");
        }

        #[submachine(machine = middle::Middle, field = middle)]
        fn mount(event: &Event) -> Outcome<State> {
            match event {
                Event::Enter(_) | Event::Ping(_) | Event::Flip(_) => Handled,
            }
        }
    }
}

#[test]
fn hooks_nest_across_awaitable_boundary() {
    let mut machine = root::Root::default().state_machine();

    block_on(machine.init(&mut ()));
    block_on(machine.handle(middle::Enter, &mut ()));
    trace::take();

    block_on(machine.handle(leaf::Ping, &mut ()));

    assert_eq!(
        trace::take(),
        [
            "root before leaf-leaf",
            "middle before leaf-leaf",
            "leaf before leaf-leaf",
            "leaf-leaf handled",
            "leaf after leaf-leaf",
            "middle after leaf-leaf",
            "root after leaf-leaf",
        ]
    );
    assert!(matches!(
        machine.state(),
        root::State::Mount(middle::State::Nesting(leaf::State::Idle {}))
    ));

    block_on(machine.handle(leaf::Flip, &mut ()));

    assert_eq!(
        trace::take(),
        [
            "root before leaf-leaf",
            "middle before leaf-leaf",
            "leaf before leaf-leaf",
            "leaf after leaf-leaf",
            "middle after leaf-leaf",
            "root after leaf-leaf",
            "root before transition",
            "middle before transition",
            "leaf before transition",
            "idle exited",
            "active entered",
            "leaf after transition",
            "middle after transition",
            "root after transition",
        ]
    );
    assert!(matches!(
        machine.state(),
        root::State::Mount(middle::State::Nesting(leaf::State::Active {}))
    ));
}
