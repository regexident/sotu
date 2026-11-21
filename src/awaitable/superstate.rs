use crate::Outcome;

use super::IntoStateMachine;

/// A superstate of the awaitable machine `M`.
///
/// A superstate groups leaf states and other superstates. It handles the
/// events its members defer to it by returning [`Outcome::Super`], and it runs
/// its entry and exit actions when a transition crosses its boundary. The
/// handler and the actions return futures that the machine awaits.
///
/// Superstates are created from the active leaf state through
/// [`State::superstate`](super::State::superstate) and may borrow that leaf's
/// local storage, so comparing two of them with [`Superstate::same_state`]
/// has to work across two independent lifetimes.
///
/// [`Superstate::superstate`] consumes the superstate and hands any borrowed
/// storage on to the parent, so only one superstate of a chain exists at any
/// time.
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
    ) -> impl Future<Output = Outcome<M::State, M::Exit>>;

    /// Consume this superstate and return the one enclosing it, if there is
    /// one.
    ///
    /// The parent has the same lifetime as `self`, so storage that `self`
    /// borrows from the active leaf state can move into the parent. The
    /// default implementation returns `None`.
    fn superstate(self) -> Option<Self>
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
}

impl<M: IntoStateMachine> Superstate<M> for () {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut M,
        _event: &M::Event,
        _context: &mut M::Context<'ctx, 'data>,
    ) -> impl Future<Output = Outcome<M::State, M::Exit>> {
        core::future::ready(Outcome::Super)
    }

    fn same_state(&self, _other: &M::Superstate<'_>) -> bool {
        true
    }
}
