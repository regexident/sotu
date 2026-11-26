#![cfg(feature = "async")]

mod common;

use common::interrupt::poll_then_drop;
use common::trace;
use futures::executor::block_on;
use proptest::prop_assert_eq;
use proptest::test_runner::TestCaseError;
use sotu::awaitable::{IntoStateMachineExt, StateMachine};
use test_strategy::proptest;

// Every action and hook of the fixtures counts a start before its only await
// and records itself after it, so the driving future can only be dropped inside
// a step that has started but is not recorded. Resuming runs that step, and
// only that step, again. A mount level creates its instance through a factory
// that runs only while the field is empty, so a repeated mount level creates
// nothing a second time.

// MARK: Probes

mod probe {
    use std::cell::Cell;

    use super::common::pending_once::pending_once;
    use super::common::trace;

    // The test harness runs each test on its own thread, so concurrent tests
    // never see each other's counts.
    std::thread_local! {
        static STARTS: Cell<u32> = const { Cell::new(0) };
        static FACTORY_CALLS: Cell<u32> = const { Cell::new(0) };
    }

    /// Return how many steps have started on the current thread since the last
    /// [`reset`].
    pub fn starts() -> u32 {
        STARTS.get()
    }

    /// Return how many factories have run on the current thread since the last
    /// [`reset`].
    pub fn factory_calls() -> u32 {
        FACTORY_CALLS.get()
    }

    pub fn reset() {
        STARTS.set(0);
        FACTORY_CALLS.set(0);
    }

    /// Count a start, suspend once and then record `entry`.
    pub async fn step(entry: &'static str) {
        STARTS.set(STARTS.get() + 1);

        pending_once().await;

        trace::record(entry);
    }

    /// Count a factory call and record that it created an instance.
    pub fn create(entry: &'static str) {
        FACTORY_CALLS.set(FACTORY_CALLS.get() + 1);

        trace::record(entry);
    }
}

// MARK: Machines

mod events {
    #[derive(Clone, Copy)]
    pub struct Enter;

    #[derive(Clone, Copy)]
    pub struct Leave;

    #[derive(Clone, Copy)]
    pub struct Start;

    #[derive(Clone, Copy)]
    pub struct Restart;

    #[derive(Clone, Copy)]
    pub struct Close;

    #[derive(Clone, Copy)]
    pub struct Ignore;

    /// The exit point of every machine.
    pub struct Finished;
}

mod door {
    use sotu::prelude::*;

    use super::events::{Close, Finished};
    use super::probe::step;
    use super::trace;

    pub struct Door;

    impl Drop for Door {
        fn drop(&mut self) {
            trace::record("door dropped");
        }
    }

    #[derive(Event)]
    pub enum Event {
        Close(Close),
    }

    #[state_machine(initial = State::opened(), exit = Finished, awaitable)]
    impl Door {
        #[state(entry_action = enter_opened, exit_action = exit_opened)]
        fn opened(event: &Event) -> Outcome<State, Finished> {
            match event {
                Event::Close(_) => Exit(Finished),
            }
        }

        #[action]
        async fn enter_opened() {
            step("door entered opened").await;
        }

        #[action]
        async fn exit_opened() {
            step("door exited opened").await;
        }
    }
}

mod line {
    use sotu::prelude::*;

    use super::events::{Close, Finished};
    use super::probe::{create, step};
    use super::{door, trace};

    pub struct Line {
        pub door: Option<door::Door>,
    }

    impl Drop for Line {
        fn drop(&mut self) {
            trace::record("line dropped");
        }
    }

    #[derive(Event)]
    pub enum Event {
        Close(Close),
    }

    #[state_machine(initial = Superstate::gate(), exit = Finished, awaitable)]
    impl Line {
        #[submachine(
            machine = door::Door,
            field = door,
            factory = Self::make_door,
            entry_action = enter_gate,
            exit_action = exit_gate,
            exit_handler = door_exited
        )]
        fn gate(event: &Event) -> Outcome<State, Finished> {
            match event {
                Event::Close(_) => Super,
            }
        }

        #[exit_handler]
        fn door_exited(exit: Finished) -> Outcome<State, Finished> {
            let Finished = exit;

            Exit(Finished)
        }

        #[action]
        async fn enter_gate() {
            step("line entered gate").await;
        }

        #[action]
        async fn exit_gate() {
            step("line exited gate").await;
        }

        fn make_door(&mut self, _context: &mut ()) -> door::Door {
            create("door created");

            door::Door
        }
    }
}

mod plant {
    use sotu::prelude::*;

    use super::events::{Close, Enter, Finished, Ignore, Leave, Restart, Start};
    use super::probe::{create, step};
    use super::{door, line};

    #[derive(Default)]
    pub struct Plant {
        pub door: Option<door::Door>,
        pub line: Option<line::Line>,
    }

    #[derive(Clone, Copy, Event)]
    pub enum Event {
        Enter(Enter),
        Leave(Leave),
        Start(Start),
        Restart(Restart),
        Close(Close),
        Ignore(Ignore),
    }

    #[state_machine(
        initial = State::idle(),
        exit = Finished,
        before_transition = Self::before_transition,
        after_transition = Self::after_transition,
        awaitable
    )]
    impl Plant {
        async fn before_transition(&mut self, _source: &State, _target: &State) {
            step("plant before transition").await;
        }

        async fn after_transition(&mut self, _source: &State, _target: &State) {
            step("plant after transition").await;
        }

        #[state(entry_action = enter_idle, exit_action = exit_idle)]
        fn idle(event: &Event) -> Outcome<State, Finished> {
            match event {
                Event::Enter(_) => Transition(Superstate::gate()),
                Event::Start(_) => Transition(Superstate::lining()),
                _ => Handled,
            }
        }

        #[submachine(
            machine = door::Door,
            field = door,
            factory = Self::make_door,
            entry_action = enter_gate,
            exit_action = exit_gate,
            exit_handler = door_exited
        )]
        fn gate(event: &Event) -> Outcome<State, Finished> {
            match event {
                Event::Leave(_) => Transition(State::idle()),
                _ => Handled,
            }
        }

        #[submachine(
            machine = line::Line,
            field = line,
            factory = Self::make_line,
            entry_action = enter_lining,
            exit_action = exit_lining,
            exit_handler = line_exited
        )]
        fn lining(event: &Event) -> Outcome<State, Finished> {
            match event {
                Event::Restart(_) => ExternalTransition(Superstate::lining()),
                _ => Handled,
            }
        }

        #[exit_handler]
        fn door_exited(exit: Finished) -> Outcome<State, Finished> {
            let Finished = exit;

            Transition(State::idle())
        }

        #[exit_handler]
        fn line_exited(exit: Finished) -> Outcome<State, Finished> {
            let Finished = exit;

            Exit(Finished)
        }

        #[action]
        async fn enter_idle() {
            step("plant entered idle").await;
        }

        #[action]
        async fn exit_idle() {
            step("plant exited idle").await;
        }

        #[action]
        async fn enter_gate() {
            step("plant entered gate").await;
        }

        #[action]
        async fn exit_gate() {
            step("plant exited gate").await;
        }

        #[action]
        async fn enter_lining() {
            step("plant entered lining").await;
        }

        #[action]
        async fn exit_lining() {
            step("plant exited lining").await;
        }

        fn make_door(&mut self, _context: &mut ()) -> door::Door {
            create("door created");

            door::Door
        }

        fn make_line(&mut self, _context: &mut ()) -> line::Line {
            create("line created");

            line::Line { door: None }
        }
    }
}

use plant::Event;

// MARK: Helpers

/// What a run leaves behind, apart from its start count.
struct Observation {
    records: Vec<&'static str>,
    starts: u32,
    factory_calls: u32,
    location: &'static str,
    door_mounted: bool,
    line_mounted: bool,
    line_door_mounted: bool,
    terminated: bool,
}

/// Return an initialized plant that has handled `setup`, with an empty trace
/// and zeroed counts.
fn running_after(setup: &[Event]) -> StateMachine<plant::Plant> {
    let mut machine = plant::Plant::default().state_machine();

    block_on(machine.init(&mut ()));

    for &event in setup {
        block_on(machine.handle(event, &mut ()));
    }

    trace::take();
    probe::reset();

    machine
}

/// Take the records out of the thread's trace and read everything else a test
/// compares from `machine`.
fn observe(machine: &StateMachine<plant::Plant>) -> Observation {
    let location = match machine.state() {
        plant::State::Idle {} => "idle",
        plant::State::Gate(_) => "gate",
        plant::State::Lining(_) => "lining",
    };

    Observation {
        records: trace::take(),
        starts: probe::starts(),
        factory_calls: probe::factory_calls(),
        location,
        door_mounted: machine.door.is_some(),
        line_mounted: machine.line.is_some(),
        line_door_mounted: machine
            .line
            .as_ref()
            .is_some_and(|line| line.door.is_some()),
        terminated: machine.is_terminated(),
    }
}

/// Interrupt `handle(event)` after `polls` polls on a plant prepared by
/// `setup`, resume it with `handle(Ignore)`, and check the outcome against an
/// uninterrupted run of the same two calls.
fn check_interrupted_handle(
    setup: &[Event],
    event: Event,
    polls: usize,
) -> Result<(), TestCaseError> {
    let mut reference = running_after(setup);

    block_on(reference.handle(event, &mut ()));
    block_on(reference.handle(Event::Ignore(events::Ignore), &mut ()));

    let expected = observe(&reference);

    let mut machine = running_after(setup);
    let before = observe(&machine);
    let completed = poll_then_drop(machine.handle(event, &mut ()), polls);

    if probe::starts() == 0 {
        // A future that was never polled leaves the machine untouched.
        let untouched = observe(&machine);

        prop_assert_eq!(&untouched.records, &before.records);
        prop_assert_eq!(untouched.starts, 0);
        prop_assert_eq!(untouched.factory_calls, 0);
        prop_assert_eq!(untouched.location, before.location);
        prop_assert_eq!(untouched.door_mounted, before.door_mounted);
        prop_assert_eq!(untouched.line_mounted, before.line_mounted);
        prop_assert_eq!(untouched.line_door_mounted, before.line_door_mounted);
        prop_assert_eq!(untouched.terminated, before.terminated);

        return Ok(());
    }

    block_on(machine.handle(Event::Ignore(events::Ignore), &mut ()));

    let actual = observe(&machine);

    // The step in flight when the future was dropped starts again, and no
    // other step does.
    let repeated = u32::from(!completed);

    prop_assert_eq!(&actual.records, &expected.records);
    prop_assert_eq!(actual.starts, expected.starts + repeated);
    prop_assert_eq!(actual.factory_calls, expected.factory_calls);
    prop_assert_eq!(actual.location, expected.location);
    prop_assert_eq!(actual.door_mounted, expected.door_mounted);
    prop_assert_eq!(actual.line_mounted, expected.line_mounted);
    prop_assert_eq!(actual.line_door_mounted, expected.line_door_mounted);
    prop_assert_eq!(actual.terminated, expected.terminated);

    Ok(())
}

// MARK: Transitions

#[proptest]
fn interrupted_entry_into_mount_resumes(#[strategy(0usize..40)] polls: usize) {
    check_interrupted_handle(&[], Event::Enter(events::Enter), polls)?;
}

#[proptest]
fn interrupted_exit_from_mount_resumes(#[strategy(0usize..40)] polls: usize) {
    check_interrupted_handle(
        &[Event::Enter(events::Enter)],
        Event::Leave(events::Leave),
        polls,
    )?;
}

#[proptest]
fn interrupted_transition_across_two_mounts_resumes(#[strategy(0usize..40)] polls: usize) {
    check_interrupted_handle(
        &[Event::Start(events::Start)],
        Event::Restart(events::Restart),
        polls,
    )?;
}

// MARK: Termination

#[proptest]
fn interrupted_termination_through_mounts_resumes(#[strategy(0usize..40)] polls: usize) {
    check_interrupted_handle(
        &[Event::Start(events::Start)],
        Event::Close(events::Close),
        polls,
    )?;
}
