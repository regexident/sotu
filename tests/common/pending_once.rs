//! A future that suspends exactly once before completing.
//!
//! Awaiting [`pending_once`] inside a handler, action or hook gives every step
//! of an awaitable machine an await point, so a test can drop the driving
//! future in the middle of that step.

use core::pin::Pin;
use core::task::{Context, Poll};

/// A future that returns [`Poll::Pending`] on its first poll and
/// [`Poll::Ready`] on every later one.
///
/// The first poll wakes the task's waker before returning, so an executor
/// polls the future again without outside help.
pub struct PendingOnce {
    polled: bool,
}

impl Future for PendingOnce {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<()> {
        if self.polled {
            return Poll::Ready(());
        }

        self.polled = true;
        context.waker().wake_by_ref();

        Poll::Pending
    }
}

/// Return a [`PendingOnce`] that has not been polled yet.
pub fn pending_once() -> PendingOnce {
    PendingOnce { polled: false }
}
