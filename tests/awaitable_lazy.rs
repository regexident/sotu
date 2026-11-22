#![cfg(feature = "async")]

mod common;

use core::convert::Infallible;

use common::async_tree::AsyncTree;
use common::tree::{Input, Node, Record};
use futures::executor::block_on;
use sotu::awaitable::{IntoStateMachine, IntoStateMachineExt, State, StateMachine};
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

// MARK: Probe Machine

#[derive(Default, Eq, PartialEq, Debug)]
struct Probe {
    value: u8,
}

#[derive(Eq, PartialEq, Debug)]
enum ProbeState {
    Only,
}

impl IntoStateMachine for Probe {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = ProbeState;
    type Superstate<'sub> = ();
    type Exit = Infallible;

    fn initial() -> ProbeState {
        ProbeState::Only
    }
}

impl State<Probe> for ProbeState {
    async fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Probe,
        _event: &(),
        _context: &mut (),
    ) -> Outcome<ProbeState, Infallible> {
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

// MARK: Conversions

#[test]
fn conversions_keep_initialization_status() {
    block_on(async {
        let mut machine = StateMachine::from(AsyncTree::default().uninitialized_state_machine());

        machine.handle(Input::Ignore, &mut ()).await;

        assert_eq!(machine.trace.first(), Some(&Record::Entry(Node::A)));

        let mut initialized = AsyncTree::default()
            .uninitialized_state_machine()
            .init(&mut ())
            .await;

        initialized.inner_mut().trace.clear();

        let mut machine = StateMachine::from(initialized);

        machine.handle(Input::Ignore, &mut ()).await;

        assert_eq!(
            machine.trace.first(),
            Some(&Record::BeforeDispatch(Node::A1))
        );
    });
}

#[test]
fn default_machine_is_uninitialized() {
    block_on(async {
        let mut machine = StateMachine::<AsyncTree>::default();

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

// MARK: Debug

#[test]
fn debug_lists_fields() {
    assert_eq!(
        format!("{:?}", Probe { value: 3 }.state_machine()),
        "StateMachine { shared_storage: Probe { value: 3 }, state: Only, exit: None, initialized: false }"
    );
}
