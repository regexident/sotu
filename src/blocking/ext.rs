use super::{IntoStateMachine, State, Superstate};

crate::hierarchy::hierarchy_algorithms!();

/// Hierarchy queries and level-wise actions for leaf states.
///
/// Implemented for every leaf state of every machine. Levels count from the
/// leaf: level `0` is the state itself and level `k` is the `k`-th superstate
/// above it.
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

    /// Run the entry action of the node `level` levels above this state.
    ///
    /// Does nothing if `level` is not smaller than the depth of this state.
    fn enter_level<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut M,
        context: &mut M::Context<'ctx, 'data>,
        level: usize,
    );

    /// Run the exit action of the node `level` levels above this state.
    ///
    /// Does nothing if `level` is not smaller than the depth of this state.
    fn exit_level<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut M,
        context: &mut M::Context<'ctx, 'data>,
        level: usize,
    );

    /// Run the entry actions of the lowest `levels` levels, outermost first.
    fn enter<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut M,
        context: &mut M::Context<'ctx, 'data>,
        levels: usize,
    );

    /// Run the exit actions of the lowest `levels` levels, innermost first.
    fn exit<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut M,
        context: &mut M::Context<'ctx, 'data>,
        levels: usize,
    );
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

    fn enter_level<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut M,
        context: &mut M::Context<'ctx, 'data>,
        level: usize,
    ) {
        if level == 0 {
            self.call_entry_action(shared_storage, context);
        } else if let Some(mut superstate) = self.superstate() {
            enter_superstate_level::<M>(&mut superstate, shared_storage, context, level - 1);
        }
    }

    fn exit_level<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut M,
        context: &mut M::Context<'ctx, 'data>,
        level: usize,
    ) {
        if level == 0 {
            self.call_exit_action(shared_storage, context);
        } else if let Some(mut superstate) = self.superstate() {
            exit_superstate_level::<M>(&mut superstate, shared_storage, context, level - 1);
        }
    }

    fn enter<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut M,
        context: &mut M::Context<'ctx, 'data>,
        levels: usize,
    ) {
        for level in (0..levels).rev() {
            self.enter_level(shared_storage, context, level);
        }
    }

    fn exit<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut M,
        context: &mut M::Context<'ctx, 'data>,
        levels: usize,
    ) {
        for level in 0..levels {
            self.exit_level(shared_storage, context, level);
        }
    }
}

/// Run the entry action of the superstate `level` levels above `superstate`.
fn enter_superstate_level<'ctx, 'data: 'ctx, M: IntoStateMachine>(
    superstate: &mut M::Superstate<'_>,
    shared_storage: &mut M,
    context: &mut M::Context<'ctx, 'data>,
    level: usize,
) {
    if level == 0 {
        superstate.call_entry_action(shared_storage, context);
    } else if let Some(mut parent) = superstate.superstate() {
        enter_superstate_level::<M>(&mut parent, shared_storage, context, level - 1);
    }
}

/// Run the exit action of the superstate `level` levels above `superstate`.
fn exit_superstate_level<'ctx, 'data: 'ctx, M: IntoStateMachine>(
    superstate: &mut M::Superstate<'_>,
    shared_storage: &mut M,
    context: &mut M::Context<'ctx, 'data>,
    level: usize,
) {
    if level == 0 {
        superstate.call_exit_action(shared_storage, context);
    } else if let Some(mut parent) = superstate.superstate() {
        exit_superstate_level::<M>(&mut parent, shared_storage, context, level - 1);
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
