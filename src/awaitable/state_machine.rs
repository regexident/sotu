use core::ops::Deref;

use super::IntoStateMachine;
use super::inner::Inner;

// MARK: Construction

/// Create awaitable state machines from their shared storage.
///
/// Implemented for every type implementing [`IntoStateMachine`].
pub trait IntoStateMachineExt: IntoStateMachine {
    /// Wrap this shared storage in a machine that has not run any action yet.
    ///
    /// The machine starts in [`IntoStateMachine::initial`].
    fn uninitialized_state_machine(self) -> UninitializedStateMachine<Self>;
}

impl<M: IntoStateMachine> IntoStateMachineExt for M {
    fn uninitialized_state_machine(self) -> UninitializedStateMachine<Self> {
        UninitializedStateMachine {
            inner: Inner::new(self),
        }
    }
}

// MARK: Uninitialized Machine

/// An awaitable state machine whose initial entry actions have not run yet.
///
/// Await [`init`](UninitializedStateMachine::init) to obtain an
/// [`InitializedStateMachine`] that can handle events.
pub struct UninitializedStateMachine<M: IntoStateMachine> {
    inner: Inner<M>,
}

impl<M: IntoStateMachine> UninitializedStateMachine<M> {
    /// Await the entry actions of every level of the initial state, outermost
    /// first, and return the running machine.
    ///
    /// No transition hook fires, because no source state exists. Dropping the
    /// returned future drops the machine with it.
    pub async fn init(mut self, context: &mut M::Context<'_, '_>) -> InitializedStateMachine<M> {
        self.inner.init(context).await;

        InitializedStateMachine { inner: self.inner }
    }

    /// Return a reference to the current leaf state.
    pub fn state(&self) -> &M::State {
        &self.inner.state
    }

    /// Return a mutable reference to the current leaf state.
    ///
    /// Changing the state in a way that violates the invariants of the
    /// hierarchy, for example by emptying an active instance field, is a logic
    /// error. It is not undefined behavior.
    pub fn state_mut(&mut self) -> &mut M::State {
        &mut self.inner.state
    }

    /// Return a reference to the shared storage.
    pub fn inner(&self) -> &M {
        &self.inner.shared_storage
    }

    /// Return a mutable reference to the shared storage.
    ///
    /// Changing the storage in a way that violates the invariants of the
    /// hierarchy, for example by emptying an active instance field, is a logic
    /// error. It is not undefined behavior.
    pub fn inner_mut(&mut self) -> &mut M {
        &mut self.inner.shared_storage
    }
}

impl<M: IntoStateMachine> Deref for UninitializedStateMachine<M> {
    type Target = M;

    fn deref(&self) -> &M {
        &self.inner.shared_storage
    }
}

// MARK: Initialized Machine

/// An awaitable state machine that has entered its initial state and handles
/// events.
pub struct InitializedStateMachine<M: IntoStateMachine> {
    inner: Inner<M>,
}

impl<M: IntoStateMachine> InitializedStateMachine<M> {
    /// Return a reference to the current leaf state.
    pub fn state(&self) -> &M::State {
        &self.inner.state
    }

    /// Return a mutable reference to the current leaf state.
    ///
    /// Changing the state in a way that violates the invariants of the
    /// hierarchy, for example by emptying an active instance field, is a logic
    /// error. It is not undefined behavior.
    pub fn state_mut(&mut self) -> &mut M::State {
        &mut self.inner.state
    }

    /// Return a reference to the shared storage.
    pub fn inner(&self) -> &M {
        &self.inner.shared_storage
    }

    /// Return a mutable reference to the shared storage.
    ///
    /// Changing the storage in a way that violates the invariants of the
    /// hierarchy, for example by emptying an active instance field, is a logic
    /// error. It is not undefined behavior.
    pub fn inner_mut(&mut self) -> &mut M {
        &mut self.inner.shared_storage
    }
}

impl<M: IntoStateMachine> Deref for InitializedStateMachine<M> {
    type Target = M;

    fn deref(&self) -> &M {
        &self.inner.shared_storage
    }
}
