use core::mem;

use crate::Outcome;

use super::{IntoStateMachine, State, StateExt};

/// The storage and execution logic shared by the machine wrappers.
///
/// `exit` is `Some` exactly when the machine has terminated.
pub(crate) struct Inner<M: IntoStateMachine> {
    pub(crate) shared_storage: M,
    pub(crate) state: M::State,
    pub(crate) exit: Option<M::Exit>,
}

impl<M: IntoStateMachine> Inner<M> {
    /// Return a machine in [`IntoStateMachine::initial`] that has run no action.
    pub(crate) fn new(shared_storage: M) -> Self {
        Self {
            shared_storage,
            state: M::initial(),
            exit: None,
        }
    }

    /// Run the entry actions of every level of the active leaf, outermost first.
    ///
    /// Runs no actions on a terminated machine.
    pub(crate) fn init(&mut self, context: &mut M::Context<'_, '_>) {
        if self.exit.is_some() {
            return;
        }

        let levels = StateExt::<M>::depth(&mut self.state);

        self.state.enter(&mut self.shared_storage, context, levels);
    }

    /// Dispatch `event` and carry out the deciding outcome.
    ///
    /// Returns the recorded exit value if this call terminated the machine. A
    /// terminated machine returns `None` without dispatching.
    pub(crate) fn handle(
        &mut self,
        event: M::Event,
        context: &mut M::Context<'_, '_>,
    ) -> Option<&M::Exit> {
        if self.exit.is_some() {
            return None;
        }

        let (outcome, handler_depth) = self.state.handle(&mut self.shared_storage, event, context);

        match outcome {
            Outcome::Handled | Outcome::Super => None,
            Outcome::Transition(mut target) => {
                let (exit_levels, enter_levels) =
                    StateExt::<M>::transition_path(&mut self.state, &mut target);

                self.transition(target, exit_levels, enter_levels, context);

                None
            }
            Outcome::ExternalTransition(mut target) => {
                let (exit_levels, enter_levels) = StateExt::<M>::external_transition_path(
                    &mut self.state,
                    &mut target,
                    handler_depth,
                );

                self.transition(target, exit_levels, enter_levels, context);

                None
            }
            Outcome::Exit(exit) => self.terminate(exit, context),
        }
    }

    /// Return a terminated machine to its initial state and initialize it again.
    ///
    /// Does nothing on a running machine.
    pub(crate) fn restart(&mut self, context: &mut M::Context<'_, '_>) {
        if self.exit.is_none() {
            return;
        }

        self.state = M::initial();
        self.exit = None;

        self.init(context);
    }

    /// Move from the active leaf to `target`.
    ///
    /// Exit actions run before the replacement, so they see the local storage of
    /// the source, and entry actions run after it, so they see that of the
    /// target. The source stays alive until the end for the hooks that observe
    /// both states.
    fn transition(
        &mut self,
        mut target: M::State,
        exit_levels: usize,
        enter_levels: usize,
        context: &mut M::Context<'_, '_>,
    ) {
        self.shared_storage
            .before_transition(&self.state, &target, context);

        let nesting = self
            .state
            .__internal_transition_nesting(&mut target, exit_levels);

        for level in 1..=nesting {
            self.state.__before_internal_transition(
                &mut target,
                exit_levels,
                level,
                &mut self.shared_storage,
                context,
            );
        }

        self.state
            .exit(&mut self.shared_storage, context, exit_levels);

        let mut source = mem::replace(&mut self.state, target);

        self.state
            .enter(&mut self.shared_storage, context, enter_levels);

        for level in (1..=nesting).rev() {
            source.__after_internal_transition(
                &mut self.state,
                exit_levels,
                level,
                &mut self.shared_storage,
                context,
            );
        }

        self.shared_storage
            .after_transition(&source, &self.state, context);
    }

    /// Leave every level of the active leaf and record `exit`.
    ///
    /// The state stays in place for inspection, and no transition hook fires
    /// because no target exists.
    fn terminate(&mut self, exit: M::Exit, context: &mut M::Context<'_, '_>) -> Option<&M::Exit> {
        let levels = StateExt::<M>::depth(&mut self.state);

        self.state.exit(&mut self.shared_storage, context, levels);
        self.exit = Some(exit);

        self.exit.as_ref()
    }
}
