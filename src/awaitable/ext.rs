use core::ops::ControlFlow;

use crate::private::{Dispatched, NodeOutcome};
use crate::{Outcome, StateOrSuperstate};

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

    /// Dispatch `event` along the active path, starting at this state.
    ///
    /// The event goes to this state and then up the chain of superstates for
    /// as long as nodes return [`Outcome::Super`]. The root's
    /// [`before_dispatch`](IntoStateMachine::before_dispatch) and
    /// [`after_dispatch`](IntoStateMachine::after_dispatch) hooks surround every
    /// dispatched node, in that order, and `after_dispatch` receives the event
    /// the node handed back. Each hook and handler is awaited before the next
    /// one starts.
    ///
    /// Returns the first outcome other than [`Outcome::Super`] together with
    /// the depth of the node that decided it, so [`Outcome::Super`] is never
    /// returned. An [`Outcome::ExternalTransition`] reports the depth of the
    /// node that handles it, which may lie above the node that returned it. A
    /// `Super` past the top-level node drops the event, which counts as handled
    /// and returns `(Outcome::Handled, 0)`.
    ///
    /// A node that reports the event as not addressed to it is skipped without
    /// hooks or handler, and the event continues upward. A node at depth `d`
    /// that defers `mount_offset` levels up skips every node deeper than
    /// `d - mount_offset` without hooks, and dispatch continues normally at
    /// that depth. An offset of `d` or more skips every remaining node and
    /// returns `(Outcome::Handled, 0)`.
    fn handle<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut M,
        event: M::Event,
        context: &mut M::Context<'ctx, 'data>,
    ) -> impl Future<Output = (Outcome<M::State, M::Exit>, usize)>;
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

    async fn handle<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut M,
        event: M::Event,
        context: &mut M::Context<'ctx, 'data>,
    ) -> (Outcome<M::State, M::Exit>, usize) {
        let mut depth = state_depth::<M>(self);
        let (is_addressed, event) = self.__is_addressed(event);

        let (mut event, mut skip_to) = if is_addressed {
            shared_storage
                .before_dispatch(StateOrSuperstate::State(&*self), &event, context)
                .await;

            let Dispatched { outcome, event } =
                self.__dispatch(shared_storage, event, context).await;

            shared_storage
                .after_dispatch(StateOrSuperstate::State(&*self), &event, context)
                .await;

            match resolve_node_outcome::<M>(outcome, depth) {
                ControlFlow::Break(decision) => return decision,
                ControlFlow::Continue(skip_to) => (event, skip_to),
            }
        } else {
            (event, None)
        };

        // Each superstate is consumed to obtain its parent once its own
        // dispatch is complete, so the walk is a loop over one owned
        // superstate and the future never has to recurse.
        let mut current = self.superstate();

        while let Some(mut superstate) = current {
            depth -= 1;

            if skip_to.is_none_or(|resume_depth| depth <= resume_depth) {
                let (is_addressed, event_to_dispatch) = superstate.__is_addressed(event);

                (event, skip_to) = if is_addressed {
                    shared_storage
                        .before_dispatch(
                            StateOrSuperstate::Superstate(&superstate),
                            &event_to_dispatch,
                            context,
                        )
                        .await;

                    let Dispatched { outcome, event } = superstate
                        .__dispatch(shared_storage, event_to_dispatch, context)
                        .await;

                    shared_storage
                        .after_dispatch(StateOrSuperstate::Superstate(&superstate), &event, context)
                        .await;

                    match resolve_node_outcome::<M>(outcome, depth) {
                        ControlFlow::Break(decision) => return decision,
                        ControlFlow::Continue(skip_to) => (event, skip_to),
                    }
                } else {
                    (event_to_dispatch, None)
                };
            }

            current = superstate.superstate();
        }

        (Outcome::Handled, 0)
    }
}

/// The deciding outcome of a dispatch and the depth of the node that decided it.
type Decision<M> = (
    Outcome<<M as IntoStateMachine>::State, <M as IntoStateMachine>::Exit>,
    usize,
);

/// Decide how dispatch proceeds after the node at `depth` returned `outcome`.
///
/// Breaks with the result of the dispatch, or continues with the depth at which
/// dispatch resumes above this node, where `None` names the parent.
fn resolve_node_outcome<M: IntoStateMachine>(
    outcome: NodeOutcome<M::State, M::Exit>,
    depth: usize,
) -> ControlFlow<Decision<M>, Option<usize>> {
    match outcome {
        NodeOutcome::Handled => ControlFlow::Break((Outcome::Handled, depth)),
        NodeOutcome::Super => ControlFlow::Continue(None),
        NodeOutcome::Transition(target) => ControlFlow::Break((Outcome::Transition(target), depth)),
        // An offset beyond the top names no handling node, which depth `0`
        // expresses to the transition path computations.
        NodeOutcome::ExternalTransition {
            target,
            handler_offset,
        } => ControlFlow::Break((
            Outcome::ExternalTransition(target),
            depth.saturating_sub(handler_offset),
        )),
        NodeOutcome::Exit(exit) => ControlFlow::Break((Outcome::Exit(exit), depth)),
        NodeOutcome::DeferToMount { mount_offset } if mount_offset >= depth => {
            ControlFlow::Break((Outcome::Handled, 0))
        }
        NodeOutcome::DeferToMount { mount_offset } => {
            ControlFlow::Continue(Some(depth - mount_offset))
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
