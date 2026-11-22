use super::{IntoStateMachine, State, Superstate};

crate::hierarchy::owned_hierarchy_algorithms!();

/// Hierarchy methods for leaf states.
///
/// Implemented for every leaf state of every awaitable machine. Levels count
/// from the leaf: level `0` is the state itself and level `k` is the `k`-th
/// superstate above it.
pub trait StateExt<M: IntoStateMachine>: State<M> {
    /// Return the depth of this state, where a top-level state has depth 1.
    fn depth(&mut self) -> usize;
}

impl<M, S> StateExt<M> for S
where
    M: IntoStateMachine<State = S>,
    S: State<M>,
{
    fn depth(&mut self) -> usize {
        state_depth::<M>(self)
    }
}

/// Hierarchy queries for superstates.
///
/// Implemented for every superstate type of every awaitable machine.
pub trait SuperstateExt<M: IntoStateMachine>: Superstate<M> {
    /// Return the depth of this superstate, where a top-level superstate has
    /// depth 1.
    ///
    /// Counting the levels above this superstate consumes it, in the same way
    /// as [`Superstate::superstate`].
    fn depth(self) -> usize;
}

// The associated type `IntoStateMachine::Superstate<'sub>` can only be named
// where `M: 'sub` holds, so the impl has to carry that bound itself.
impl<'sub, M, S> SuperstateExt<M> for S
where
    M: IntoStateMachine<Superstate<'sub> = S>,
    M: 'sub,
    S: Superstate<M>,
{
    fn depth(self) -> usize {
        superstate_depth::<M>(self)
    }
}
