#![cfg(feature = "async")]

mod common;

use core::mem;

use common::async_tree::{AsyncTree, AsyncTreeState, reset_starts, starts};
use common::interrupt::poll_then_drop;
use common::tree::{Finished, Input, Leaf, Record};
use futures::executor::block_on;
use proptest::prop_assert_eq;
use proptest::test_runner::TestCaseError;
use sotu::awaitable::{InitializedStateMachine, IntoStateMachineExt, StateMachine};
use test_strategy::proptest;

// Every step of `AsyncTree` counts its start before its only await, so the
// driving future can only be dropped inside a step that has started. When a
// future is dropped after at least one poll, exactly one step is therefore
// started but unrecorded. Resuming an initialization, transition or
// termination runs that step, and only that step, again. An interrupted
// dispatch step never runs again, because the event is lost.

// MARK: Helpers

/// Return an initialized machine that has handled `events`, with an empty
/// trace and the start count reset.
fn running_after(events: &[Input]) -> InitializedStateMachine<AsyncTree> {
    let mut machine = block_on(
        AsyncTree::default()
            .uninitialized_state_machine()
            .init(&mut ()),
    );

    for &event in events {
        block_on(machine.handle(event, &mut ()));
    }

    machine.inner_mut().trace.clear();
    reset_starts();

    machine
}

/// Take the records out of `trace` and return them with the start count of
/// the current thread, which is reset to zero.
fn take_run(trace: &mut Vec<Record>) -> (Vec<Record>, u32) {
    let run = (mem::take(trace), starts());

    reset_starts();

    run
}

fn is_dispatch(record: &Record) -> bool {
    matches!(
        record,
        Record::BeforeDispatch(_) | Record::Handler(_) | Record::AfterDispatch(_)
    )
}

/// Interrupt `handle(event)` after `polls` polls on a machine prepared by
/// `setup`, resume it with `handle(Input::Ignore)`, and check the outcome
/// against uninterrupted reference runs.
///
/// Returns the resumed machine and whether the interruption lost the event,
/// which happens exactly when no step after the dispatch had started.
fn check_interrupted_handle(
    setup: &[Input],
    event: Input,
    polls: usize,
) -> Result<(InitializedStateMachine<AsyncTree>, bool), TestCaseError> {
    // The operation and a following `Ignore`, both uninterrupted.
    let mut reference = running_after(setup);
    block_on(reference.handle(event, &mut ()));
    let (records, step_starts) = take_run(&mut reference.inner_mut().trace);
    block_on(reference.handle(Input::Ignore, &mut ()));
    let (ignore_records, ignore_starts) = take_run(&mut reference.inner_mut().trace);

    // Only an `Ignore`, as seen by a machine that lost `event`.
    let mut lost_reference = running_after(setup);
    block_on(lost_reference.handle(Input::Ignore, &mut ()));
    let (lost_ignore_records, lost_ignore_starts) = take_run(&mut lost_reference.inner_mut().trace);

    let dispatch_steps = records
        .iter()
        .take_while(|record| is_dispatch(record))
        .count();

    let mut machine = running_after(setup);
    let completed = poll_then_drop(machine.handle(event, &mut ()), polls);
    let started = starts();

    if completed {
        prop_assert_eq!(&machine.trace, &records);
        prop_assert_eq!(started, step_starts);
        prop_assert_eq!(machine.state(), reference.state());
        prop_assert_eq!(machine.exit(), reference.exit());

        return Ok((machine, false));
    }

    block_on(machine.handle(Input::Ignore, &mut ()));

    let lost = started as usize <= dispatch_steps;

    if lost {
        // The dispatch steps before the interrupted one completed, and nothing
        // resumes the interrupted one.
        let completed_steps = (started as usize).saturating_sub(1);
        let expected_records = [&records[..completed_steps], &lost_ignore_records[..]].concat();

        prop_assert_eq!(&machine.trace, &expected_records);
        prop_assert_eq!(starts(), started + lost_ignore_starts);
        prop_assert_eq!(machine.state(), lost_reference.state());
        prop_assert_eq!(machine.exit(), lost_reference.exit());
    } else {
        let expected_records = [&records[..], &ignore_records[..]].concat();

        prop_assert_eq!(&machine.trace, &expected_records);
        prop_assert_eq!(starts(), step_starts + 1 + ignore_starts);
        prop_assert_eq!(machine.state(), reference.state());
        prop_assert_eq!(machine.exit(), reference.exit());
    }

    Ok((machine, lost))
}

// MARK: Initialization

#[proptest]
fn interrupted_initialization_resumes(#[strategy(0usize..12)] polls: usize) {
    let mut reference: StateMachine<AsyncTree> = AsyncTree::default().state_machine();
    reset_starts();
    block_on(reference.init(&mut ()));
    let (records, step_starts) = take_run(&mut reference.inner_mut().trace);

    let mut machine: StateMachine<AsyncTree> = AsyncTree::default().state_machine();
    let completed = poll_then_drop(machine.init(&mut ()), polls);
    let started = starts();

    if completed {
        prop_assert_eq!(&machine.trace, &records);
        prop_assert_eq!(started, step_starts);
    } else {
        block_on(machine.init(&mut ()));

        // An unpolled future started nothing, so nothing runs twice.
        let repeated = u32::from(started > 0);

        prop_assert_eq!(&machine.trace, &records);
        prop_assert_eq!(starts(), step_starts + repeated);
    }

    prop_assert_eq!(machine.state(), reference.state());
}

// MARK: Transition

#[proptest]
fn interrupted_transition_resumes(#[strategy(0usize..24)] polls: usize) {
    let (machine, lost) = check_interrupted_handle(&[], Input::Go(Leaf::B1), polls)?;

    let expected_state = if lost {
        AsyncTreeState::A1
    } else {
        AsyncTreeState::B1 { visits: 2 }
    };

    prop_assert_eq!(machine.state(), &expected_state);
}

// MARK: Termination

#[proptest]
fn interrupted_termination_resumes(#[strategy(0usize..12)] polls: usize) {
    let (machine, lost) = check_interrupted_handle(&[Input::Go(Leaf::B1)], Input::Finish, polls)?;

    let expected_exit = if lost { None } else { Some(&Finished::Done) };

    prop_assert_eq!(machine.exit(), expected_exit);
    prop_assert_eq!(machine.is_terminated(), !lost);
}

// MARK: Dispatch

#[test]
fn interrupted_dispatch_loses_event() {
    let mut reference = running_after(&[]);
    block_on(reference.handle(Input::Ignore, &mut ()));
    let (ignore_records, ignore_starts) = take_run(&mut reference.inner_mut().trace);

    let mut machine = running_after(&[]);

    // The first poll suspends inside the machine's `before_dispatch` hook for
    // `A1`, before the handler or any transition step starts.
    assert!(!poll_then_drop(
        machine.handle(Input::Go(Leaf::B1), &mut ()),
        1
    ));
    assert_eq!(starts(), 1);
    assert_eq!(machine.trace, []);
    assert_eq!(machine.state(), &AsyncTreeState::A1);

    reset_starts();
    block_on(machine.handle(Input::Ignore, &mut ()));

    assert_eq!(machine.trace, ignore_records);
    assert_eq!(starts(), ignore_starts);
    assert_eq!(machine.state(), &AsyncTreeState::A1);
    assert!(!machine.is_terminated());
}
