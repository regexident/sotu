mod common;

use core::convert::Infallible;

use common::tree::{Input, Node, Record, Tree};
use sotu::blocking::{IntoStateMachine, IntoStateMachineExt, State, StateMachine};
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
    fn call_handler<'ctx, 'data: 'ctx>(
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
    let mut machine = Tree::default().state_machine();

    machine.handle(Input::Ignore, &mut ());

    assert_eq!(
        machine.trace[..3],
        [
            Record::Entry(Node::A),
            Record::Entry(Node::A1),
            Record::BeforeDispatch(Node::A1),
        ]
    );
}

#[test]
fn init_is_idempotent() {
    let mut machine = Tree::default().state_machine();

    machine.init(&mut ());
    machine.init(&mut ());

    assert_eq!(
        machine.trace,
        [Record::Entry(Node::A), Record::Entry(Node::A1)]
    );
}

// MARK: Conversions

#[test]
fn conversion_from_uninitialized_stays_uninitialized() {
    let mut machine = StateMachine::from(Tree::default().uninitialized_state_machine());

    machine.handle(Input::Ignore, &mut ());

    assert_eq!(machine.trace.first(), Some(&Record::Entry(Node::A)));
}

#[test]
fn conversion_from_initialized_stays_initialized() {
    let mut initialized = Tree::default().uninitialized_state_machine().init(&mut ());

    initialized.inner_mut().trace.clear();

    let mut machine = StateMachine::from(initialized);

    machine.handle(Input::Ignore, &mut ());

    assert_eq!(
        machine.trace.first(),
        Some(&Record::BeforeDispatch(Node::A1))
    );
}

#[test]
fn default_machine_is_uninitialized() {
    let mut machine = StateMachine::<Tree>::default();

    machine.init(&mut ());

    assert_eq!(
        machine.trace,
        [Record::Entry(Node::A), Record::Entry(Node::A1)]
    );
}

// MARK: Unit Events

#[test]
fn step_dispatches_unit_event() {
    let mut machine = Ticker::default().state_machine();

    machine.step(&mut ());
    machine.step(&mut ());

    assert_eq!(machine.ticks, 2);

    let mut initialized = Ticker::default()
        .uninitialized_state_machine()
        .init(&mut ());

    initialized.step(&mut ());

    assert_eq!(initialized.ticks, 1);
}

// MARK: Restart

#[test]
fn restart_on_uninitialized_machine_does_nothing() {
    let mut machine = Tree::default().state_machine();

    machine.restart(&mut ());

    assert_eq!(machine.trace, []);
}
