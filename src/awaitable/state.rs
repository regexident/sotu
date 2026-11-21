use crate::Outcome;

use super::IntoStateMachine;

/// A leaf state of the awaitable machine `M`.
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
    ) -> impl Future<Output = Outcome<Self, M::Exit>>;

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
}
