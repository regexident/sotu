use core::mem;

use crate::Outcome;

use super::{IntoStateMachine, State, StateExt};

/// The storage and execution logic shared by the awaitable machine wrappers.
///
/// `exit` is `Some` exactly when the machine has terminated. `pending` records
/// how far an initialization, transition or termination has progressed, so a
/// future that is dropped while it awaits an action or hook leaves a record
/// from which the next call resumes the work.
pub(crate) struct Inner<M: IntoStateMachine> {
    pub(crate) shared_storage: M,
    pub(crate) state: M::State,
    pub(crate) exit: Option<M::Exit>,
    /// The state replaced by a transition, kept until its after-hooks ran.
    source: Option<M::State>,
    pending: Option<Pending<M::State, M::Exit>>,
}

/// The progress of an initialization, transition or termination whose actions
/// and hooks have not all completed.
///
/// Every counter counts completed calls. A call that was interrupted counts as
/// not completed and runs again from its start.
pub(crate) enum Pending<S, X> {
    /// Entering the levels of the active leaf, outermost first.
    Init { entered: usize, levels: usize },
    /// Moving from the active leaf to `target`.
    ///
    /// `target` is `Some` until all `exit_levels` levels are exited. The state
    /// then holds the target and the source moves to [`Inner`]'s `source`.
    Transition {
        target: Option<S>,
        exit_levels: usize,
        exited: usize,
        enter_levels: usize,
        entered: usize,
        nesting: usize,
        phase: Phase,
    },
    /// Leaving every level of the active leaf before recording `exit`.
    Terminate {
        exit: X,
        exited: usize,
        levels: usize,
    },
}

/// The sub-step of a pending transition.
pub(crate) enum Phase {
    /// Calling the root's `before_transition` hook as call `0`, then the
    /// internal hooks for the nesting levels `1..=nesting` as calls
    /// `1..=nesting`.
    BeforeHook { completed: usize },
    /// Exiting the levels of the source, innermost first.
    Exiting,
    /// Entering the levels of the target, outermost first.
    Entering,
    /// Calling the internal hooks for the nesting levels `nesting..=1`, then
    /// the root's `after_transition` hook.
    AfterHook { completed: usize },
}

impl<M: IntoStateMachine> Inner<M> {
    /// Return a machine in [`IntoStateMachine::initial`] that has run no action.
    pub(crate) fn new(shared_storage: M) -> Self {
        Self {
            shared_storage,
            state: M::initial(),
            exit: None,
            source: None,
            pending: None,
        }
    }

    /// Complete any pending work, then await the entry actions of every level
    /// of the active leaf, outermost first.
    ///
    /// Runs no actions on a terminated machine.
    pub(crate) async fn init(&mut self, context: &mut M::Context<'_, '_>) {
        self.drive_pending(context).await;

        if self.exit.is_some() {
            return;
        }

        self.begin_init();
        self.drive_pending(context).await;
    }

    /// Complete any pending work, then dispatch `event` and carry out the
    /// deciding outcome.
    ///
    /// Returns the recorded exit value if this call's event terminated the
    /// machine. A terminated machine returns `None` without dispatching, and so
    /// does a call that only completed a pending termination.
    ///
    /// No record exists while the event is dispatched, so an interrupted
    /// dispatch loses the event and leaves nothing to resume.
    pub(crate) async fn handle(
        &mut self,
        event: M::Event,
        context: &mut M::Context<'_, '_>,
    ) -> Option<&M::Exit> {
        self.drive_pending(context).await;

        if self.exit.is_some() {
            return None;
        }

        let (outcome, handler_depth) =
            StateExt::<M>::handle(&mut self.state, &mut self.shared_storage, event, context).await;

        match outcome {
            Outcome::Handled | Outcome::Super => None,
            Outcome::Transition(mut target) => {
                let (exit_levels, enter_levels) =
                    StateExt::<M>::transition_path(&mut self.state, &mut target);

                self.begin_transition(target, exit_levels, enter_levels);
                self.drive_pending(context).await;

                None
            }
            Outcome::ExternalTransition(mut target) => {
                let (exit_levels, enter_levels) = StateExt::<M>::external_transition_path(
                    &mut self.state,
                    &mut target,
                    handler_depth,
                );

                self.begin_transition(target, exit_levels, enter_levels);
                self.drive_pending(context).await;

                None
            }
            Outcome::Exit(exit) => {
                let levels = StateExt::<M>::depth(&mut self.state);

                self.pending = Some(Pending::Terminate {
                    exit,
                    exited: 0,
                    levels,
                });
                self.drive_pending(context).await;

                self.exit.as_ref()
            }
        }
    }

    /// Complete any pending work, then return a terminated machine to its
    /// initial state and initialize it again.
    ///
    /// Does nothing else on a running machine.
    pub(crate) async fn restart(&mut self, context: &mut M::Context<'_, '_>) {
        self.drive_pending(context).await;

        if self.exit.is_none() {
            return;
        }

        self.state = M::initial();
        self.exit = None;

        self.begin_init();
        self.drive_pending(context).await;
    }

    /// Record an initialization of every level of the active leaf.
    fn begin_init(&mut self) {
        let levels = StateExt::<M>::depth(&mut self.state);

        self.pending = Some(Pending::Init { entered: 0, levels });
    }

    /// Record a transition from the active leaf to `target`.
    fn begin_transition(&mut self, mut target: M::State, exit_levels: usize, enter_levels: usize) {
        let nesting = self
            .state
            .__internal_transition_nesting(&mut target, exit_levels);

        self.pending = Some(Pending::Transition {
            target: Some(target),
            exit_levels,
            exited: 0,
            enter_levels,
            entered: 0,
            nesting,
            phase: Phase::BeforeHook { completed: 0 },
        });
    }

    /// Carry the pending work out to completion, one action or hook at a time.
    ///
    /// Each iteration either awaits exactly one action or hook and then counts
    /// it as completed, or advances the record without awaiting. The record
    /// therefore never claims a call that has not completed, and a call that is
    /// interrupted runs again from its start on the next drive.
    pub(crate) async fn drive_pending(&mut self, context: &mut M::Context<'_, '_>) {
        while let Some(pending) = &mut self.pending {
            match pending {
                Pending::Init { entered, levels } => {
                    if *entered < *levels {
                        let level = *levels - 1 - *entered;

                        self.state
                            .enter_level(&mut self.shared_storage, context, level)
                            .await;
                        *entered += 1;
                    } else {
                        self.pending = None;
                    }
                }
                Pending::Transition {
                    target,
                    exit_levels,
                    exited,
                    enter_levels,
                    entered,
                    nesting,
                    phase,
                } => match phase {
                    Phase::BeforeHook { completed } if *completed == 0 => {
                        // The target stays in the record until every exit
                        // action completed, which follows the before-hooks.
                        if let Some(target) = target {
                            self.shared_storage
                                .before_transition(&self.state, target, context)
                                .await;
                        }

                        *completed += 1;
                    }
                    Phase::BeforeHook { completed } if *completed <= *nesting => {
                        if let Some(target) = target {
                            self.state
                                .__before_internal_transition(
                                    target,
                                    *exit_levels,
                                    *completed,
                                    &mut self.shared_storage,
                                    context,
                                )
                                .await;
                        }

                        *completed += 1;
                    }
                    Phase::BeforeHook { .. } => *phase = Phase::Exiting,
                    Phase::Exiting if *exited < *exit_levels => {
                        self.state
                            .exit_level(&mut self.shared_storage, context, *exited)
                            .await;
                        *exited += 1;
                    }
                    Phase::Exiting => {
                        // Exit actions saw the source, and entry actions see
                        // the target, so the swap sits between the two.
                        if let Some(target) = target.take() {
                            self.source = Some(mem::replace(&mut self.state, target));
                        }

                        *phase = Phase::Entering;
                    }
                    Phase::Entering if *entered < *enter_levels => {
                        let level = *enter_levels - 1 - *entered;

                        self.state
                            .enter_level(&mut self.shared_storage, context, level)
                            .await;
                        *entered += 1;
                    }
                    Phase::Entering => *phase = Phase::AfterHook { completed: 0 },
                    Phase::AfterHook { completed } if *completed < *nesting => {
                        // The source left the state in the swap, which
                        // precedes the after-hooks.
                        if let Some(source) = &mut self.source {
                            source
                                .__after_internal_transition(
                                    &mut self.state,
                                    *exit_levels,
                                    *nesting - *completed,
                                    &mut self.shared_storage,
                                    context,
                                )
                                .await;
                        }

                        *completed += 1;
                    }
                    Phase::AfterHook { completed } if *completed == *nesting => {
                        if let Some(source) = &self.source {
                            self.shared_storage
                                .after_transition(source, &self.state, context)
                                .await;
                        }

                        *completed += 1;
                    }
                    Phase::AfterHook { .. } => {
                        self.source = None;
                        self.pending = None;
                    }
                },
                Pending::Terminate { exited, levels, .. } => {
                    if *exited < *levels {
                        self.state
                            .exit_level(&mut self.shared_storage, context, *exited)
                            .await;
                        *exited += 1;
                    } else if let Some(Pending::Terminate { exit, .. }) = self.pending.take() {
                        self.exit = Some(exit);
                    }
                }
            }
        }
    }
}
