use super::{IntoStateMachine, State, Superstate};

crate::hierarchy::hierarchy_algorithms!();

/// Hierarchy queries for leaf states.
///
/// Implemented for every leaf state of every machine.
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
