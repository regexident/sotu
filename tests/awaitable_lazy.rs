#![cfg(feature = "async")]

mod common;

use core::convert::Infallible;

use common::async_tree::AsyncTree;
use common::tree::{Input, Node, Record};
use futures::executor::block_on;
use sotu::awaitable::{IntoStateMachine, IntoStateMachineExt, State};
use sotu::prelude::Outcome;

// MARK: Ticker Machine

// A flat machine whose only event is the tick `()`, the shape `step` exists for.

#[derive(Default)]
struct Ticker {
    ticks: u32,
}

enum TickerState {
    Running,
}

impl IntoStateMachine for Ticker {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = TickerState;
    type Superstate<'sub> = ();
    type Exit = Infallible;

    fn initial() -> TickerState {
        TickerState::Running
    }
}

impl State<Ticker> for TickerState {
    async fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Ticker,
        _event: &(),
        _context: &mut (),
    ) -> Outcome<TickerState, Infallible> {
        shared_storage.ticks += 1;

        Outcome::Handled
    }
}

// MARK: Lazy Initialization

#[test]
fn first_handle_initializes_before_dispatch() {
    block_on(async {
        let mut machine = AsyncTree::default().state_machine();

        machine.handle(Input::Ignore, &mut ()).await;

        assert_eq!(
            machine.trace[..3],
            [
                Record::Entry(Node::A),
                Record::Entry(Node::A1),
                Record::BeforeDispatch(Node::A1),
            ]
        );
    });
}

#[test]
fn init_is_idempotent() {
    block_on(async {
        let mut machine = AsyncTree::default().state_machine();

        machine.init(&mut ()).await;
        machine.init(&mut ()).await;

        assert_eq!(
            machine.trace,
            [Record::Entry(Node::A), Record::Entry(Node::A1)]
        );
    });
}

// MARK: Unit Events

#[test]
fn step_dispatches_unit_event() {
    block_on(async {
        let mut machine = Ticker::default().state_machine();

        machine.step(&mut ()).await;
        machine.step(&mut ()).await;

        assert_eq!(machine.ticks, 2);
    });
}
