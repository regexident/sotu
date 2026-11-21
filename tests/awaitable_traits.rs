#![cfg(feature = "async")]

mod common;

use core::pin::pin;
use core::task::{Context, Poll, Waker};

use common::pending_once::pending_once;

// MARK: Test Futures

#[test]
fn pending_once_yields_exactly_once() {
    let mut future = pin!(pending_once());
    let mut context = Context::from_waker(Waker::noop());

    assert_eq!(future.as_mut().poll(&mut context), Poll::Pending);
    assert_eq!(future.as_mut().poll(&mut context), Poll::Ready(()));
}
