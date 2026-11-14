//! A log of the steps that fixture machines take, kept per thread.
//!
//! The test harness runs each test on its own thread, so tests that record
//! steps concurrently never see each other's entries.

use std::cell::RefCell;

std::thread_local! {
    static ENTRIES: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
}

/// Append `entry` to the log of the current thread.
pub fn record(entry: &'static str) {
    ENTRIES.with_borrow_mut(|entries| entries.push(entry));
}

/// Return the entries recorded on the current thread, leaving its log empty.
pub fn take() -> Vec<&'static str> {
    ENTRIES.with_borrow_mut(std::mem::take)
}
