//! Composable hierarchical state machines.
//!
//! `sotu` provides hierarchical state machines in which states are grouped
//! under superstates and unhandled events defer to the enclosing node. The
//! crate is `no_std` and never allocates.
//!
//! A machine can mount other machines, including machines defined in other
//! modules or crates, so large systems are assembled from independently
//! written parts.

#![no_std]
#![forbid(unsafe_code)]

pub mod event;
mod node;
mod outcome;

pub use node::{StateOrSuperstate, SubmachineNode};
pub use outcome::Outcome;

/// Common imports for writing state machines.
pub mod prelude {
    pub use crate::node::{StateOrSuperstate, SubmachineNode};
    pub use crate::outcome::Outcome::{self, *};
}
