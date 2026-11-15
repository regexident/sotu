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

mod dryer {
    use sotu::prelude::*;

    use super::{door, trace};

    #[derive(Default)]
    pub struct Dryer {
        pub door: Option<door::Door>,
    }

    pub struct Start;

    pub struct Tick;

    #[derive(Event)]
    pub enum Event {
        Start(Start),
        Tick(Tick),
        Open(door::Open),
        Close(door::Close),
    }

    #[state_machine(
        initial = State::idle(),
        before_dispatch = Self::before,
        after_dispatch = Self::after
    )]
    impl Dryer {
        fn before(&mut self, node: StateOrSuperstate<'_, State, Superstate<'_>>, _event: &Event) {
            trace::record(match node {
                StateOrSuperstate::State(State::Idle {}) => "root before idle",
                StateOrSuperstate::State(State::Loading(_)) => "root before leaf",
                StateOrSuperstate::Superstate(Superstate::Loading(SubmachineNode::Inner(_))) => {
                    "root before child superstate"
                }
                StateOrSuperstate::Superstate(Superstate::Loading(SubmachineNode::Mount)) => {
                    "root before loading"
                }
            });
        }

        fn after(&mut self, node: StateOrSuperstate<'_, State, Superstate<'_>>, _event: &Event) {
            trace::record(match node {
                StateOrSuperstate::State(State::Idle {}) => "root after idle",
                StateOrSuperstate::State(State::Loading(_)) => "root after leaf",
                StateOrSuperstate::Superstate(Superstate::Loading(SubmachineNode::Inner(_))) => {
                    "root after child superstate"
                }
                StateOrSuperstate::Superstate(Superstate::Loading(SubmachineNode::Mount)) => {
                    "root after loading"
                }
            });
        }

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
                Event::Tick(_) => "loading saw tick",
                Event::Open(_) => "loading saw open",
                Event::Close(_) => "loading saw close",
            });

            Handled
        }
    }
}

/// Return an initialized `Dryer` in `loading` with the child in `opened`,
/// with an empty trace.
fn loading_dryer() -> InitializedStateMachine<dryer::Dryer> {
    let mut machine = dryer::Dryer::default()
        .uninitialized_state_machine()
        .init(&mut ());

    machine.handle(dryer::Start, &mut ());
    trace::take();

    machine
}

#[test]
fn unlisted_event_skips_child_without_hooks() {
    let mut machine = loading_dryer();

    machine.handle(dryer::Tick, &mut ());

    assert_eq!(
        trace::take(),
        [
            "root before loading",
            "loading saw tick",
            "root after loading"
        ]
    );
    assert!(matches!(
        machine.state(),
        dryer::State::Loading(door::State::Opened {})
    ));
}

#[test]
fn child_super_returns_event_to_mount_handler() {
    let mut machine = loading_dryer();

    machine.handle(door::Open, &mut ());

    assert_eq!(
        trace::take(),
        [
            "root before leaf",
            "opened deferred open",
            "root after leaf",
            "root before child superstate",
            "frame deferred open",
            "root after child superstate",
            "root before loading",
            "loading saw open",
            "root after loading",
        ]
    );
}

use test_strategy::{Arbitrary, proptest};

mod counter {
    use sotu::prelude::*;

    #[derive(Default)]
    pub struct Counter {
        pub count: u32,
    }

    #[state_machine(initial = State::counting())]
    impl Counter {
        #[state]
        fn counting(&mut self) -> Outcome<State> {
            self.count += 1;

            Handled
        }
    }
}

mod ticker {
    use sotu::prelude::*;

    use super::counter;

    #[derive(Default)]
    pub struct Ticker {
        pub counter: Option<counter::Counter>,
    }

    #[state_machine(initial = Superstate::running())]
    impl Ticker {
        #[submachine(machine = counter::Counter, field = counter)]
        fn running() -> Outcome<State> {
            Handled
        }
    }
}

#[test]
fn unit_child_receives_every_step() {
    let mut machine = ticker::Ticker::default()
        .uninitialized_state_machine()
        .init(&mut ());

    machine.step(&mut ());
    machine.step(&mut ());
    machine.step(&mut ());

    assert_eq!(
        machine.counter.as_ref().map(|counter| counter.count),
        Some(3)
    );
}

/// An event of `Dryer`, generated without the payload types having to
/// implement `Arbitrary`.
#[derive(Copy, Clone, Debug, Arbitrary)]
enum DryerInput {
    Start,
    Tick,
    Open,
    Close,
}

impl DryerInput {
    fn into_event(self) -> dryer::Event {
        match self {
            Self::Start => dryer::Event::Start(dryer::Start),
            Self::Tick => dryer::Event::Tick(dryer::Tick),
            Self::Open => dryer::Event::Open(door::Open),
            Self::Close => dryer::Event::Close(door::Close),
        }
    }
}

/// The entries that the handlers of `door::Door` record.
const DOOR_HANDLER_ENTRIES: [&str; 6] = [
    "opened handled close",
    "opened deferred open",
    "closed handled open",
    "closed deferred close",
    "frame deferred open",
    "frame deferred close",
];

#[proptest]
fn addressing_matches_listed_types(input: DryerInput) {
    let mut machine = loading_dryer();

    machine.handle(input.into_event(), &mut ());

    let entries = trace::take();
    let reaches_child = entries
        .iter()
        .any(|entry| DOOR_HANDLER_ENTRIES.contains(entry));
    let is_listed = matches!(input, DryerInput::Open | DryerInput::Close);

    proptest::prop_assert_eq!(reaches_child, is_listed);
}

#[test]
fn child_transition_is_lifted_and_keeps_instance() {
    let mut machine = washer::Washer::default()
        .uninitialized_state_machine()
        .init(&mut ());

    trace::take();
    machine.handle(washer::Start, &mut ());
    machine.handle(door::Close, &mut ());
    machine.handle(door::Open, &mut ());

    assert_eq!(
        trace::take(),
        [
            "door created",
            "opened handled close",
            "closed handled open"
        ]
    );
    assert!(matches!(
        machine.state(),
        washer::State::Loading(door::State::Opened {})
    ));
    assert!(machine.door.is_some());
}

mod shutter {
    use sotu::prelude::*;

    use super::trace;

    pub struct Shutter;

    impl Default for Shutter {
        fn default() -> Self {
            trace::record("shutter created");

            Self
        }
    }

    impl Drop for Shutter {
        fn drop(&mut self) {
            trace::record("shutter dropped");
        }
    }

    pub struct Reset;

    #[derive(Event)]
    pub enum Event {
        Reset(Reset),
    }

    #[state_machine(initial = State::closed())]
    impl Shutter {
        #[state(superstate = frame, entry_action = enter_opened, exit_action = exit_opened)]
        fn opened(event: &Event) -> Outcome<State> {
            match event {
                Event::Reset(_) => trace::record("opened deferred reset"),
            }

            Super
        }

        #[state(superstate = frame, entry_action = enter_closed, exit_action = exit_closed)]
        fn closed(event: &Event) -> Outcome<State> {
            match event {
                Event::Reset(_) => trace::record("closed deferred reset"),
            }

            Super
        }

        #[superstate(entry_action = enter_frame, exit_action = exit_frame)]
        fn frame(event: &Event) -> Outcome<State> {
            match event {
                Event::Reset(_) => trace::record("frame handled reset"),
            }

            ExternalTransition(State::opened())
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

mod booth {
    use sotu::prelude::*;

    use super::{shutter, trace};

    #[derive(Default)]
    pub struct Booth {
        pub shutter: Option<shutter::Shutter>,
    }

    #[derive(Event)]
    pub enum Event {
        Reset(shutter::Reset),
    }

    #[state_machine(initial = Superstate::framing())]
    impl Booth {
        #[submachine(machine = shutter::Shutter, field = shutter)]
        fn framing(event: &Event) -> Outcome<State> {
            match event {
                Event::Reset(_) => trace::record("framing saw reset"),
            }

            Handled
        }
    }
}

#[test]
fn child_external_transition_reenters_child_superstate_only() {
    let mut machine = booth::Booth::default()
        .uninitialized_state_machine()
        .init(&mut ());

    assert_eq!(
        trace::take(),
        ["shutter created", "enter frame", "enter closed"]
    );

    machine.handle(shutter::Reset, &mut ());

    assert_eq!(
        trace::take(),
        [
            "closed deferred reset",
            "frame handled reset",
            "exit closed",
            "exit frame",
            "enter frame",
            "enter opened",
        ]
    );
    assert!(matches!(
        machine.state(),
        booth::State::Framing(shutter::State::Opened {})
    ));
    assert!(machine.shutter.is_some());
}

#[test]
fn missing_instance_defers_to_mount_handler() {
    let mut machine = loading_dryer();

    machine.inner_mut().door = None;

    assert_eq!(trace::take(), ["door dropped"]);

    machine.handle(door::Open, &mut ());

    assert_eq!(
        trace::take(),
        [
            "root before leaf",
            "root after leaf",
            "root before loading",
            "loading saw open",
            "root after loading",
        ]
    );
    assert!(machine.door.is_none());
}
