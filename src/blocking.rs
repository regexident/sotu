//! Blocking state machines.
//!
//! A machine is a type implementing [`IntoStateMachine`], which holds the
//! machine's shared storage. Its leaf states implement [`State`] and its
//! superstates implement [`Superstate`]. Handlers run to completion on the
//! calling thread and never suspend.

mod into_state_machine;
mod state;
mod superstate;

pub use into_state_machine::IntoStateMachine;
pub use state::State;
pub use superstate::Superstate;
