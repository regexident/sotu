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

pub mod blocking;
pub mod event;
mod hierarchy;
mod node;
mod outcome;
#[doc(hidden)]
pub mod private;

pub use node::{StateOrSuperstate, SubmachineNode};
pub use outcome::Outcome;
#[cfg(feature = "macros")]
pub use sotu_macros::Event;

/// Common imports for writing state machines.
pub mod prelude {
    pub use crate::blocking::{
        InitializedStateMachine, IntoStateMachine, State, StateMachine, Superstate,
        UninitializedStateMachine,
    };
    pub use crate::blocking::{IntoStateMachineExt as _, StateExt as _, SuperstateExt as _};
    pub use crate::node::{StateOrSuperstate, SubmachineNode};
    pub use crate::outcome::Outcome::{self, *};

    #[cfg(feature = "macros")]
    pub use sotu_macros::Event;
}
