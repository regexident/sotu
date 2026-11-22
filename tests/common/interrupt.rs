//! Drop a future part-way through its execution.
//!
//! Polling with [`Waker::noop`] and then dropping the future simulates a
//! caller that abandons an awaitable operation, for example on a timeout, at
//! an exact await point.

use core::pin::pin;
use core::task::{Context, Waker};

/// Poll `future` at most `polls` times, then drop it.
///
/// Returns `true` if the future completed within `polls` polls and `false` if
/// it was dropped while still pending. With `polls` set to `0` the future is
/// dropped without ever being polled.
pub fn poll_then_drop<F: Future>(future: F, polls: usize) -> bool {
    let mut future = pin!(future);
    let mut context = Context::from_waker(Waker::noop());

    for _ in 0..polls {
        if future.as_mut().poll(&mut context).is_ready() {
            return true;
        }
    }

    false
}
