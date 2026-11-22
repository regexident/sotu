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

    /// Await the entry action of the node `level` levels above this state.
    ///
    /// Does nothing if `level` is not smaller than the depth of this state.
    fn enter_level<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut M,
        context: &mut M::Context<'ctx, 'data>,
        level: usize,
    ) -> impl Future<Output = ()>;

    /// Await the exit action of the node `level` levels above this state.
    ///
    /// Does nothing if `level` is not smaller than the depth of this state.
    fn exit_level<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut M,
        context: &mut M::Context<'ctx, 'data>,
        level: usize,
    ) -> impl Future<Output = ()>;

    /// Await the entry actions of the lowest `levels` levels, outermost
    /// first.
    fn enter<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut M,
        context: &mut M::Context<'ctx, 'data>,
        levels: usize,
    ) -> impl Future<Output = ()>;

    /// Await the exit actions of the lowest `levels` levels, innermost first.
    fn exit<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut M,
        context: &mut M::Context<'ctx, 'data>,
        levels: usize,
    ) -> impl Future<Output = ()>;
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

    async fn enter_level<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut M,
        context: &mut M::Context<'ctx, 'data>,
        level: usize,
    ) {
        if level == 0 {
            self.call_entry_action(shared_storage, context).await;
        } else if let Some(mut superstate) = superstate_at_level::<M>(self, level) {
            superstate.call_entry_action(shared_storage, context).await;
        }
    }

    async fn exit_level<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut M,
        context: &mut M::Context<'ctx, 'data>,
        level: usize,
    ) {
        if level == 0 {
            self.call_exit_action(shared_storage, context).await;
        } else if let Some(mut superstate) = superstate_at_level::<M>(self, level) {
            superstate.call_exit_action(shared_storage, context).await;
        }
    }

    async fn enter<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut M,
        context: &mut M::Context<'ctx, 'data>,
        levels: usize,
    ) {
        for level in (0..levels).rev() {
            self.enter_level(shared_storage, context, level).await;
        }
    }

    async fn exit<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut M,
        context: &mut M::Context<'ctx, 'data>,
        levels: usize,
    ) {
        for level in 0..levels {
            self.exit_level(shared_storage, context, level).await;
        }
    }
}

/// Return the superstate `level` levels above `state`, or `None` if the
/// hierarchy above `state` has fewer levels.
///
/// `level` must be at least 1. The chain is stepped synchronously before any
/// action is awaited, so the returned superstate is the only one alive.
fn superstate_at_level<M: IntoStateMachine>(
    state: &mut M::State,
    level: usize,
) -> Option<M::Superstate<'_>> {
    let mut superstate = state.superstate()?;

    for _ in 1..level {
        superstate = superstate.superstate()?;
    }

    Some(superstate)
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
