#![cfg(feature = "async")]

mod common;

use core::convert::Infallible;

use common::pending_once::pending_once;
use futures::executor::block_on;
use sotu::StateOrSuperstate;
use sotu::awaitable::TransitionHook;
use sotu::awaitable::{DispatchHook, IntoStateMachine, State};
use sotu::prelude::Outcome;

// MARK: Probe Machine

struct Counter {
    calls: u32,
}

enum CounterState {
    Only,
}

impl IntoStateMachine for Counter {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = u32;
    type State = CounterState;
    type Superstate<'sub> = ();
    type Exit = Infallible;

    fn initial() -> CounterState {
        CounterState::Only
    }
}

impl State<Counter> for CounterState {
    async fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Counter,
        _event: &(),
        _context: &mut u32,
    ) -> Outcome<CounterState, Infallible> {
        Outcome::Handled
    }
}

// MARK: Dispatch Hooks

fn count(counter: &mut Counter, _node: StateOrSuperstate<'_, CounterState, ()>, _event: &()) {
    counter.calls += 1;
}

fn count_with_context(
    counter: &mut Counter,
    _node: StateOrSuperstate<'_, CounterState, ()>,
    _event: &(),
    context: &mut u32,
) {
    counter.calls += 1;
    *context += 1;
}

async fn count_async(
    counter: &mut Counter,
    _node: StateOrSuperstate<'_, CounterState, ()>,
    _event: &(),
) {
    pending_once().await;
    counter.calls += 1;
}

async fn count_async_with_context(
    counter: &mut Counter,
    _node: StateOrSuperstate<'_, CounterState, ()>,
    _event: &(),
    context: &mut u32,
) {
    pending_once().await;
    counter.calls += 1;
    *context += 1;
}

#[test]
fn plain_dispatch_hook_without_context() {
    let mut counter = Counter { calls: 0 };
    let mut context = 5;

    block_on(DispatchHook::call_dispatch_hook(
        &count,
        &mut counter,
        StateOrSuperstate::State(&CounterState::Only),
        &(),
        &mut context,
    ));

    assert_eq!(counter.calls, 1);
    assert_eq!(context, 5);
}

#[test]
fn plain_dispatch_hook_with_context() {
    let mut counter = Counter { calls: 0 };
    let mut context = 5;

    block_on(DispatchHook::call_dispatch_hook(
        &count_with_context,
        &mut counter,
        StateOrSuperstate::State(&CounterState::Only),
        &(),
        &mut context,
    ));

    assert_eq!(counter.calls, 1);
    assert_eq!(context, 6);
}

#[test]
fn async_dispatch_hook_without_context() {
    let mut counter = Counter { calls: 0 };
    let mut context = 5;

    block_on(DispatchHook::call_dispatch_hook(
        &count_async,
        &mut counter,
        StateOrSuperstate::State(&CounterState::Only),
        &(),
        &mut context,
    ));

    assert_eq!(counter.calls, 1);
    assert_eq!(context, 5);
}

#[test]
fn async_dispatch_hook_with_context() {
    let mut counter = Counter { calls: 0 };
    let mut context = 5;

    block_on(DispatchHook::call_dispatch_hook(
        &count_async_with_context,
        &mut counter,
        StateOrSuperstate::State(&CounterState::Only),
        &(),
        &mut context,
    ));

    assert_eq!(counter.calls, 1);
    assert_eq!(context, 6);
}

// MARK: Transition Hooks

fn count_transition(counter: &mut Counter, _source: &CounterState, _target: &CounterState) {
    counter.calls += 1;
}

fn count_transition_with_context(
    counter: &mut Counter,
    _source: &CounterState,
    _target: &CounterState,
    context: &mut u32,
) {
    counter.calls += 1;
    *context += 1;
}

async fn count_async_transition(
    counter: &mut Counter,
    _source: &CounterState,
    _target: &CounterState,
) {
    pending_once().await;
    counter.calls += 1;
}

async fn count_async_transition_with_context(
    counter: &mut Counter,
    _source: &CounterState,
    _target: &CounterState,
    context: &mut u32,
) {
    pending_once().await;
    counter.calls += 1;
    *context += 1;
}

#[test]
fn plain_transition_hook_without_context() {
    let mut counter = Counter { calls: 0 };
    let mut context = 5;

    block_on(TransitionHook::call_transition_hook(
        &count_transition,
        &mut counter,
        &CounterState::Only,
        &CounterState::Only,
        &mut context,
    ));

    assert_eq!(counter.calls, 1);
    assert_eq!(context, 5);
}

#[test]
fn plain_transition_hook_with_context() {
    let mut counter = Counter { calls: 0 };
    let mut context = 5;

    block_on(TransitionHook::call_transition_hook(
        &count_transition_with_context,
        &mut counter,
        &CounterState::Only,
        &CounterState::Only,
        &mut context,
    ));

    assert_eq!(counter.calls, 1);
    assert_eq!(context, 6);
}

#[test]
fn async_transition_hook_without_context() {
    let mut counter = Counter { calls: 0 };
    let mut context = 5;

    block_on(TransitionHook::call_transition_hook(
        &count_async_transition,
        &mut counter,
        &CounterState::Only,
        &CounterState::Only,
        &mut context,
    ));

    assert_eq!(counter.calls, 1);
    assert_eq!(context, 5);
}

#[test]
fn async_transition_hook_with_context() {
    let mut counter = Counter { calls: 0 };
    let mut context = 5;

    block_on(TransitionHook::call_transition_hook(
        &count_async_transition_with_context,
        &mut counter,
        &CounterState::Only,
        &CounterState::Only,
        &mut context,
    ));

    assert_eq!(counter.calls, 1);
    assert_eq!(context, 6);
}
