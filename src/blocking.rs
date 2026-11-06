//! Blocking state machines.
//!
//! A machine is a type implementing [`IntoStateMachine`], which holds the
//! machine's shared storage. Its leaf states implement [`State`] and its
//! superstates implement [`Superstate`]. Handlers run to completion on the
//! calling thread and never suspend.
//!
//! [`IntoStateMachineExt::uninitialized_state_machine`] wraps the shared storage
//! in an [`UninitializedStateMachine`], and its `init` method returns the
//! [`InitializedStateMachine`] that handles events.
//! [`IntoStateMachineExt::state_machine`] wraps it in a [`StateMachine`] instead,
//! which initializes itself on first use.

mod ext;
mod hook;
mod inner;
mod into_state_machine;
mod state;
mod state_machine;
mod superstate;

pub use ext::{StateExt, SuperstateExt};
pub use hook::{DispatchHook, TransitionHook};
pub use into_state_machine::IntoStateMachine;
pub use state::State;
pub use state_machine::{
    InitializedStateMachine, IntoStateMachineExt, StateMachine, UninitializedStateMachine,
};
pub use superstate::Superstate;
