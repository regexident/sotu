use super::{IntoStateMachine, State, Superstate};

crate::hierarchy::hierarchy_algorithms!();

/// Hierarchy queries for leaf states.
///
/// Implemented for every leaf state of every machine.
pub trait StateExt<M: IntoStateMachine>: State<M> {
    /// Return the depth of this state, where a top-level state has depth 1.
    fn depth(&mut self) -> usize;

    /// Return how many levels a local transition from `self` to `target`
    /// exits and enters, as `(exit_levels, enter_levels)`.
    fn transition_path(&mut self, target: &mut Self) -> (usize, usize);

    /// Return how many levels an external transition from `self` to `target`
    /// exits and enters when the node at `handler_depth` handles it, as
    /// `(exit_levels, enter_levels)`.
    ///
    /// A `handler_depth` of `0`, or one greater than the depth of `self`,
    /// names no handling node and yields the same result as
    /// [`transition_path`](StateExt::transition_path).
    fn external_transition_path(
        &mut self,
        target: &mut Self,
        handler_depth: usize,
    ) -> (usize, usize);
}

impl<M, S> StateExt<M> for S
where
    M: IntoStateMachine<State = S>,
    S: State<M>,
{
    fn depth(&mut self) -> usize {
        state_depth::<M>(self)
    }

    fn transition_path(&mut self, target: &mut Self) -> (usize, usize) {
        transition_path::<M>(self, target)
    }

    fn external_transition_path(
        &mut self,
        target: &mut Self,
        handler_depth: usize,
    ) -> (usize, usize) {
        external_transition_path::<M>(self, target, handler_depth)
    }
}

/// Hierarchy queries for superstates.
///
/// Implemented for every superstate type of every machine.
pub trait SuperstateExt<M: IntoStateMachine>: Superstate<M> {
    /// Return the depth of this superstate, where a top-level superstate has
    /// depth 1.
    fn depth(&mut self) -> usize;
}

// The associated type `IntoStateMachine::Superstate<'sub>` can only be named
// where `M::State: 'sub` holds, so the impl has to carry that bound itself.
impl<'sub, M, S> SuperstateExt<M> for S
where
    M: IntoStateMachine<Superstate<'sub> = S>,
    M::State: 'sub,
    S: Superstate<M>,
{
    fn depth(&mut self) -> usize {
        superstate_depth::<M>(self)
    }
}
