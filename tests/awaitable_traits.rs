#![cfg(feature = "async")]

mod common;

use core::convert::Infallible;
use core::pin::pin;
use core::task::{Context, Poll, Waker};

use common::async_tree::{AsyncTree, AsyncTreeState, AsyncTreeSuperstate};
use common::pending_once::pending_once;
use common::tree::{Input, Leaf, Node, Record};
use futures::executor::block_on;
use sotu::Outcome;
use sotu::awaitable::{IntoStateMachine, State, Superstate};
use sotu::private::{Dispatched, NodeOutcome};

// MARK: Probe Machine

#[derive(Eq, PartialEq, Debug)]
struct Probe;

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

// MARK: Superstate Defaults

#[test]
fn unit_superstate_defers_and_matches_itself() {
    assert_eq!(
        block_on(<() as Superstate<Probe>>::call_handler(
            &mut (),
            &mut Probe,
            &(),
            &mut ()
        )),
        Outcome::Super
    );
    assert!(<() as Superstate<Probe>>::same_state(&(), &()));
    assert!(<() as Superstate<Probe>>::superstate(()).is_none());
}

// MARK: State Defaults

#[test]
fn same_state_compares_variants_only() {
    assert!(AsyncTreeState::B1 { visits: 1 }.same_state(&AsyncTreeState::B1 { visits: 2 }));
    assert!(!AsyncTreeState::B1 { visits: 0 }.same_state(&AsyncTreeState::B2 { visits: 0 }));
}

#[test]
fn default_dispatch_awaits_handler_and_returns_event() {
    let mut tree = AsyncTree::default();

    let dispatched =
        block_on(AsyncTreeState::A1.__dispatch(&mut tree, Input::Go(Leaf::C), &mut ()));

    assert_eq!(
        dispatched,
        Dispatched {
            outcome: NodeOutcome::Transition(AsyncTreeState::C),
            event: Input::Go(Leaf::C),
        }
    );
    assert_eq!(tree.trace, vec![Record::Handler(Node::A1)]);
}

// MARK: Superstate Chains

#[test]
fn consuming_superstate_moves_borrowed_storage() {
    let mut state = AsyncTreeState::B1 { visits: 0 };

    match state.superstate() {
        Some(AsyncTreeSuperstate::B { visits }) => *visits = 5,
        other => panic!("expected B, found {other:?}"),
    }

    assert_eq!(state, AsyncTreeState::B1 { visits: 5 });

    let mut value = 0;

    assert!(matches!(
        AsyncTreeSuperstate::B { visits: &mut value }.superstate(),
        Some(AsyncTreeSuperstate::A)
    ));
}

// MARK: Test Futures

#[test]
fn pending_once_yields_exactly_once() {
    let mut future = pin!(pending_once());
    let mut context = Context::from_waker(Waker::noop());

    assert_eq!(future.as_mut().poll(&mut context), Poll::Pending);
    assert_eq!(future.as_mut().poll(&mut context), Poll::Ready(()));
}
