#![cfg(feature = "async")]

use core::convert::Infallible;

use futures::executor::block_on;
use sotu::StateOrSuperstate;
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
