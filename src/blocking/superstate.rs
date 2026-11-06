use crate::Outcome;
use crate::private::{Dispatched, NodeOutcome};

use super::IntoStateMachine;

/// A superstate of the machine `M`.
///
/// A superstate groups leaf states and other superstates. It handles the
/// events its members defer to it by returning [`Outcome::Super`], and it runs
/// its entry and exit actions when a transition crosses its boundary.
///
/// Superstates are created from the active leaf state through
/// [`State::superstate`](super::State::superstate) and may borrow that leaf's
/// local storage, so comparing two of them with [`Superstate::same_state`]
/// has to work across two independent lifetimes.
///
/// The unit type is a superstate that defers every event and matches only
/// itself, for hierarchies that need a placeholder at the top.
pub trait Superstate<M: IntoStateMachine> {
    /// Handle `event` and decide what the machine does next.
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut M,
        event: &M::Event,
        context: &mut M::Context<'ctx, 'data>,
    ) -> Outcome<M::State, M::Exit>;

    /// Run when the machine enters this superstate.
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut M,
        _context: &mut M::Context<'ctx, 'data>,
    ) {
    }

    /// Run when the machine leaves this superstate.
    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut M,
        _context: &mut M::Context<'ctx, 'data>,
    ) {
    }

    /// Return the superstate enclosing this one, if there is one.
    ///
    /// The default implementation returns `None`.
    fn superstate(&mut self) -> Option<M::Superstate<'_>>
    where
        Self: Sized,
    {
        None
    }

    /// Return whether `self` and `other` are the same superstate.
    ///
    /// `other` may borrow from a different lifetime than `self`, which is why
    /// the method takes it through [`IntoStateMachine::Superstate`] instead of
    /// `Self`.
    fn same_state(&self, other: &M::Superstate<'_>) -> bool;

    #[doc(hidden)]
    fn __is_addressed(&self, event: M::Event) -> (bool, M::Event) {
        (true, event)
    }

    #[doc(hidden)]
    fn __dispatch<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut M,
        event: M::Event,
        context: &mut M::Context<'ctx, 'data>,
    ) -> Dispatched<M::State, M::Exit, M::Event> {
        let outcome = self.call_handler(shared_storage, &event, context);

        Dispatched {
            outcome: NodeOutcome::from(outcome),
            event,
        }
    }
}

impl<M: IntoStateMachine> Superstate<M> for () {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut M,
        _event: &M::Event,
        _context: &mut M::Context<'ctx, 'data>,
    ) -> Outcome<M::State, M::Exit> {
        Outcome::Super
    }

    fn same_state(&self, _other: &M::Superstate<'_>) -> bool {
        true
    }
}
