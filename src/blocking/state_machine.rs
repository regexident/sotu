use core::ops::Deref;

use super::IntoStateMachine;
use super::inner::Inner;

// MARK: Construction

/// Create state machines from their shared storage.
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

/// A state machine whose initial entry actions have not run yet.
///
/// Call [`init`](UninitializedStateMachine::init) to obtain an
/// [`InitializedStateMachine`] that can handle events.
pub struct UninitializedStateMachine<M: IntoStateMachine> {
    inner: Inner<M>,
}

impl<M: IntoStateMachine> UninitializedStateMachine<M> {
    /// Run the entry actions of every level of the initial state, outermost
    /// first, and return the running machine.
    ///
    /// No transition hook fires, because no source state exists.
    pub fn init(mut self, context: &mut M::Context<'_, '_>) -> InitializedStateMachine<M> {
        self.inner.init(context);

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

/// A state machine that has entered its initial state and handles events.
///
/// The machine terminates when an [`Outcome::Exit`](crate::Outcome::Exit)
/// reaches it. A terminated machine keeps its last state for inspection and
/// ignores events until [`restart`](InitializedStateMachine::restart) is called.
pub struct InitializedStateMachine<M: IntoStateMachine> {
    inner: Inner<M>,
}

impl<M: IntoStateMachine> InitializedStateMachine<M> {
    /// Dispatch `event` along the active path and perform the resulting
    /// transition or termination.
    ///
    /// Returns the exit value if this call terminated the machine, borrowed from
    /// the recorded value that [`exit`](InitializedStateMachine::exit) also
    /// returns. Returns `None` otherwise, and in particular on a terminated
    /// machine, which does not dispatch the event at all.
    ///
    /// A transition runs the root's `before_transition` hook, the exit actions
    /// innermost first on the old state, the replacement of the state, the entry
    /// actions outermost first on the new state, and the root's
    /// `after_transition` hook. Termination runs the exit actions of every level
    /// of the active leaf, innermost first, and fires no transition hook.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::convert::Infallible;
    ///
    /// use sotu::prelude::*;
    ///
    /// #[derive(Default)]
    /// struct Light {
    ///     toggles: u32,
    /// }
    ///
    /// #[derive(Eq, PartialEq, Debug)]
    /// enum LightState {
    ///     Off,
    ///     On,
    /// }
    ///
    /// impl IntoStateMachine for Light {
    ///     type Event = ();
    ///     type Context<'ctx, 'data: 'ctx> = ();
    ///     type State = LightState;
    ///     type Superstate<'sub> = ();
    ///     type Exit = Infallible;
    ///
    ///     fn initial() -> LightState {
    ///         LightState::Off
    ///     }
    /// }
    ///
    /// impl State<Light> for LightState {
    ///     fn call_handler<'ctx, 'data: 'ctx>(
    ///         &mut self,
    ///         shared_storage: &mut Light,
    ///         _event: &(),
    ///         _context: &mut (),
    ///     ) -> Outcome<LightState, Infallible> {
    ///         shared_storage.toggles += 1;
    ///
    ///         match self {
    ///             LightState::Off => Outcome::Transition(LightState::On),
    ///             LightState::On => Outcome::Transition(LightState::Off),
    ///         }
    ///     }
    /// }
    ///
    /// let mut machine = Light::default().uninitialized_state_machine().init(&mut ());
    ///
    /// assert_eq!(machine.handle((), &mut ()), None);
    /// assert_eq!(machine.state(), &LightState::On);
    /// assert_eq!(machine.toggles, 1);
    /// ```
    pub fn handle(
        &mut self,
        event: impl Into<M::Event>,
        context: &mut M::Context<'_, '_>,
    ) -> Option<&M::Exit> {
        self.inner.handle(event.into(), context)
    }

    /// Reset a terminated machine to [`IntoStateMachine::initial`], clear its
    /// exit value and run the initial entry actions again.
    ///
    /// Does nothing on a running machine. The shared storage is kept.
    pub fn restart(&mut self, context: &mut M::Context<'_, '_>) {
        self.inner.restart(context);
    }

    /// Return whether the machine has terminated.
    pub fn is_terminated(&self) -> bool {
        self.inner.exit.is_some()
    }

    /// Return the exit value of a terminated machine, or `None` while it runs.
    pub fn exit(&self) -> Option<&M::Exit> {
        self.inner.exit.as_ref()
    }

    /// Consume the machine and return its exit value, or `None` if it has not
    /// terminated.
    pub fn into_exit(self) -> Option<M::Exit> {
        self.inner.exit
    }

    /// Return a reference to the current leaf state.
    ///
    /// On a terminated machine this is the last active state.
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
