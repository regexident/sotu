//! Awaitable state machines.
//!
//! A machine is a type implementing [`IntoStateMachine`], which holds the
//! machine's shared storage. Its leaf states implement [`State`] and its
//! superstates implement [`Superstate`]. These traits mirror their
//! counterparts in [`blocking`](crate::blocking), except that every handler,
//! action and hook returns a [`Future`] that the machine awaits. A step that
//! awaits can suspend, and dropping the future that drives the machine cancels
//! the step at its current await point.
//!
//! This mode is available with the `async` feature. It needs neither an
//! allocator nor a particular executor, and the returned futures carry no
//! `Send` bound.
//!
//! [`IntoStateMachineExt::uninitialized_state_machine`] wraps the shared
//! storage in an [`UninitializedStateMachine`], and awaiting its `init` method
//! returns the [`InitializedStateMachine`] that handles events.
//! [`IntoStateMachineExt::state_machine`] wraps it in a [`StateMachine`]
//! instead, which initializes itself on first use. The prelude brings the
//! extension trait into scope but not the wrapper types, whose names equal
//! those of the blocking wrappers.
//!
//! A blocking machine never implements these traits, so a type is a machine
//! of exactly one mode and method calls through the prelude stay unambiguous.
//!
//! # Superstate chains
//!
//! [`Superstate::superstate`] differs from its blocking counterpart: it takes
//! the superstate by value and returns the parent with the same lifetime,
//! moving any storage borrowed from the active leaf state into the parent.
//! The blocking method borrows the superstate mutably, so walking up the
//! hierarchy holds one borrow per level. Awaiting while such a chain of
//! borrows is alive would take a recursive future, and a recursive future has
//! to be boxed, which requires an allocator. Consuming each superstate keeps
//! exactly one superstate alive at a time.
//!
//! [`State::superstate`] still borrows the leaf state, because the leaf
//! outlives every superstate built from it.

mod ext;
mod hook;
mod inner;
mod into_state_machine;
mod state;
mod state_machine;
mod superstate;

pub use ext::StateExt;
pub use ext::SuperstateExt;
pub use hook::DispatchHook;
pub use hook::TransitionHook;
pub use into_state_machine::IntoStateMachine;
pub use state::State;
pub use state_machine::InitializedStateMachine;
pub use state_machine::IntoStateMachineExt;
pub use state_machine::StateMachine;
pub use state_machine::UninitializedStateMachine;
pub use superstate::Superstate;
