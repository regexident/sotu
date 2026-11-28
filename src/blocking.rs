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
//!
//! Use this mode when every handler and action can finish without waiting,
//! which includes machines on embedded targets that are driven from an
//! interrupt or a main loop. Prefer the awaitable mode behind the `async`
//! feature when a handler or action has to suspend, for example to wait for a
//! peripheral. Both modes share the macro syntax, the [`Outcome`](crate::Outcome)
//! type and the event model.
//!
//! # The traits
//!
//! `#[state_machine]` implements [`IntoStateMachine`] for the annotated type,
//! and [`State`] and [`Superstate`] for the enums it generates. Builds without
//! the `macros` feature implement them by hand.
//!
//! - [`IntoStateMachine`] names the event, context, state, superstate and exit
//!   types of a machine and provides its initial state. It also carries the
//!   four hooks, which observe dispatch and transitions and default to doing
//!   nothing.
//! - [`State`] and [`Superstate`] hold the handler, the entry and exit actions
//!   and the link to the enclosing node. A leaf state creates its superstate on
//!   demand and lends its local storage to it.
//! - [`StateExt`] and [`SuperstateExt`] add the hierarchy walks that the
//!   machine runs on every event: the depth of a node, the levels a transition
//!   exits and enters, and the dispatch of an event up the chain of nodes.
//! - [`DispatchHook`] and [`TransitionHook`] let generated code call a hook
//!   function whether or not it takes the context.
//!
//! # Running a machine
//!
//! The three wrappers differ in when the initial entry actions run.
//! [`UninitializedStateMachine`] has run nothing and offers no `handle`, so a
//! machine that handles events has always been initialized.
//! [`InitializedStateMachine`] is what `init` returns. [`StateMachine`] checks
//! on every `handle` call whether it has been initialized, which spares the
//! caller a separate step at the cost of that check.
//!
//! An event goes to the active leaf and then up the chain of superstates for as
//! long as nodes return `Super`. The first other outcome decides what happens.
//! A transition runs the `before_transition` hook, the exit actions innermost
//! first, replaces the state, runs the entry actions outermost first and then
//! runs the `after_transition` hook. An `Exit` runs the exit actions of every
//! active level and terminates the machine, which then ignores events until it
//! is restarted.
//!
//! A machine without handlers that take an event has the event type `()`,
//! which is the tick event, and is driven with `step` instead of `handle`.
//!
//! # Examples
//!
//! A countdown that arms itself on entry and terminates when it reaches zero:
//!
//! ```
//! use sotu::prelude::*;
//!
//! struct Countdown {
//!     remaining: u32,
//!     armed: bool,
//! }
//!
//! #[derive(PartialEq, Debug)]
//! struct Liftoff;
//!
//! #[state_machine(initial = State::counting(), exit = Liftoff)]
//! impl Countdown {
//!     #[state(entry_action = arm)]
//!     fn counting(&mut self) -> Outcome<State, Liftoff> {
//!         self.remaining = self.remaining.saturating_sub(1);
//!
//!         if self.remaining == 0 {
//!             Exit(Liftoff)
//!         } else {
//!             Handled
//!         }
//!     }
//!
//!     #[action]
//!     fn arm(&mut self) {
//!         self.armed = true;
//!     }
//! }
//!
//! let machine = Countdown {
//!     remaining: 2,
//!     armed: false,
//! }
//! .uninitialized_state_machine();
//!
//! assert!(!machine.armed);
//!
//! let mut machine = machine.init(&mut ());
//!
//! assert!(machine.armed);
//! assert_eq!(machine.step(&mut ()), None);
//! assert_eq!(machine.step(&mut ()), Some(&Liftoff));
//! assert!(machine.is_terminated());
//!
//! // A terminated machine ignores events and keeps its storage.
//! assert_eq!(machine.step(&mut ()), None);
//! assert_eq!(machine.remaining, 0);
//!
//! machine.restart(&mut ());
//!
//! assert!(!machine.is_terminated());
//! ```

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
