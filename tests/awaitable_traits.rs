#![cfg(feature = "async")]

mod common;

use core::convert::Infallible;
use core::pin::pin;
use core::task::{Context, Poll, Waker};

use common::pending_once::pending_once;
use futures::executor::block_on;
use sotu::Outcome;
use sotu::awaitable::{IntoStateMachine, State, Superstate};

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

// MARK: Test Futures

#[test]
fn pending_once_yields_exactly_once() {
    let mut future = pin!(pending_once());
    let mut context = Context::from_waker(Waker::noop());

    assert_eq!(future.as_mut().poll(&mut context), Poll::Pending);
    assert_eq!(future.as_mut().poll(&mut context), Poll::Ready(()));
}
