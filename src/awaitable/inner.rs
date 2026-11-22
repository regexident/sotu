use super::{IntoStateMachine, StateExt};

/// The storage and execution logic shared by the awaitable machine wrappers.
///
/// `pending` records how far an initialization has progressed, one completed
/// entry action at a time.
pub(crate) struct Inner<M: IntoStateMachine> {
    pub(crate) shared_storage: M,
    pub(crate) state: M::State,
    pending: Option<Pending>,
}

/// The progress of an initialization whose entry actions have not all
/// completed.
///
/// Every counter counts completed calls. A call that was interrupted counts as
/// not completed and runs again from its start.
pub(crate) enum Pending {
    /// Entering the levels of the active leaf, outermost first.
    Init { entered: usize, levels: usize },
}

impl<M: IntoStateMachine> Inner<M> {
    /// Return a machine in [`IntoStateMachine::initial`] that has run no action.
    pub(crate) fn new(shared_storage: M) -> Self {
        Self {
            shared_storage,
            state: M::initial(),
            pending: None,
        }
    }

    /// Await the entry actions of every level of the active leaf, outermost
    /// first.
    pub(crate) async fn init(&mut self, context: &mut M::Context<'_, '_>) {
        self.begin_init();
        self.drive_pending(context).await;
    }

    /// Record an initialization of every level of the active leaf.
    fn begin_init(&mut self) {
        let levels = StateExt::<M>::depth(&mut self.state);

        self.pending = Some(Pending::Init { entered: 0, levels });
    }

    /// Carry the pending work out to completion, one action at a time.
    ///
    /// Each iteration either awaits exactly one action and then counts it as
    /// completed, or advances the record without awaiting. The record
    /// therefore never claims a call that has not completed, and a call that is
    /// interrupted runs again from its start on the next drive.
    async fn drive_pending(&mut self, context: &mut M::Context<'_, '_>) {
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
            }
        }
    }
}
