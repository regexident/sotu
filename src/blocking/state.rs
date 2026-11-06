use crate::Outcome;
use crate::private::{Dispatched, NodeOutcome};

use super::IntoStateMachine;

/// A leaf state of the machine `M`.
///
/// Leaf states compare by variant only: [`State::same_state`] ignores the
/// values of any local storage a state carries. Two states with different
/// local storage therefore count as the same state when they are the same
/// variant.
///
/// A state that sits inside a superstate returns it from
/// [`State::superstate`]. Events a state does not handle are deferred to that
/// superstate by returning [`Outcome::Super`].
pub trait State<M: IntoStateMachine>: Sized {
    /// Handle `event` and decide what the machine does next.
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut M,
        event: &M::Event,
        context: &mut M::Context<'ctx, 'data>,
    ) -> Outcome<Self, M::Exit>;

    /// Run when the machine enters this state.
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut M,
        _context: &mut M::Context<'ctx, 'data>,
    ) {
    }

    /// Run when the machine leaves this state.
    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut M,
        _context: &mut M::Context<'ctx, 'data>,
    ) {
    }

    /// Return the superstate enclosing this state, if there is one.
    ///
    /// The superstate may borrow this state's local storage. The default
    /// implementation returns `None`.
    fn superstate(&mut self) -> Option<M::Superstate<'_>> {
        None
    }

    /// Return whether `self` and `other` are the same leaf state.
    ///
    /// The default implementation compares variants and ignores local storage
    /// values.
    fn same_state(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }

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
    ) -> Dispatched<Self, M::Exit, M::Event> {
        let outcome = self.call_handler(shared_storage, &event, context);

        Dispatched {
            outcome: NodeOutcome::from(outcome),
            event,
        }
    }

    #[doc(hidden)]
    fn __internal_transition_nesting(&mut self, _target: &mut Self, _exit_levels: usize) -> usize {
        0
    }

    #[doc(hidden)]
    fn __before_internal_transition<'ctx, 'data: 'ctx>(
        &mut self,
        _target: &mut Self,
        _exit_levels: usize,
        _nesting: usize,
        _shared_storage: &mut M,
        _context: &mut M::Context<'ctx, 'data>,
    ) {
    }

    #[doc(hidden)]
    fn __after_internal_transition<'ctx, 'data: 'ctx>(
        &mut self,
        _target: &mut Self,
        _exit_levels: usize,
        _nesting: usize,
        _shared_storage: &mut M,
        _context: &mut M::Context<'ctx, 'data>,
    ) {
    }
}
