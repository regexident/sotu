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

    /// Wrap this shared storage in a machine that initializes itself on first
    /// use.
    ///
    /// The machine starts in [`IntoStateMachine::initial`] and has not run any
    /// action yet.
    fn state_machine(self) -> StateMachine<Self>;
}

impl<M: IntoStateMachine> IntoStateMachineExt for M {
    fn uninitialized_state_machine(self) -> UninitializedStateMachine<Self> {
        UninitializedStateMachine {
            inner: Inner::new(self),
        }
    }

    fn state_machine(self) -> StateMachine<Self> {
        StateMachine {
            inner: Inner::new(self),
            initialized: false,
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
///
/// The machine terminates when an [`Outcome::Exit`](crate::Outcome::Exit)
/// reaches it. A terminated machine keeps its last state for inspection and
/// ignores events until [`restart`](InitializedStateMachine::restart) is
/// awaited.
///
/// # Cancellation
///
/// The machine records its progress after every completed action and hook of
/// an initialization, a transition or a termination. If the future of
/// [`handle`](InitializedStateMachine::handle),
/// [`step`](InitializedStateMachine::step) or
/// [`restart`](InitializedStateMachine::restart) is dropped while it awaits one
/// of them, the next of these calls first completes the interrupted work. The
/// interrupted action or hook runs again from its start, and every completed
/// one is never repeated.
///
/// Dropping the future while the event is being dispatched, that is while a
/// handler or a dispatch hook is awaited, loses the event: nothing is resumed
/// for it.
pub struct InitializedStateMachine<M: IntoStateMachine> {
    inner: Inner<M>,
}

impl<M: IntoStateMachine> InitializedStateMachine<M> {
    /// Complete any interrupted work, then dispatch `event` along the active
    /// path and perform the resulting transition or termination.
    ///
    /// Returns the exit value if this call's event terminated the machine,
    /// borrowed from the recorded value that
    /// [`exit`](InitializedStateMachine::exit) also returns. Returns `None`
    /// otherwise, in particular on a terminated machine, which does not
    /// dispatch the event at all, and when the call only completed an
    /// interrupted termination.
    ///
    /// A transition awaits the root's `before_transition` hook, the exit
    /// actions innermost first on the old state, the replacement of the state,
    /// the entry actions outermost first on the new state, and the root's
    /// `after_transition` hook. Termination awaits the exit actions of every
    /// level of the active leaf, innermost first, and fires no transition hook.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::convert::Infallible;
    ///
    /// use futures::executor::block_on;
    /// use sotu::awaitable::{IntoStateMachine, State};
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
    ///     async fn call_handler<'ctx, 'data: 'ctx>(
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
    /// block_on(async {
    ///     let mut machine = Light::default()
    ///         .uninitialized_state_machine()
    ///         .init(&mut ())
    ///         .await;
    ///
    ///     assert_eq!(machine.handle((), &mut ()).await, None);
    ///     assert_eq!(machine.state(), &LightState::On);
    ///     assert_eq!(machine.toggles, 1);
    /// });
    /// ```
    pub async fn handle(
        &mut self,
        event: impl Into<M::Event>,
        context: &mut M::Context<'_, '_>,
    ) -> Option<&M::Exit> {
        self.inner.handle(event.into(), context).await
    }

    /// Dispatch the unit event `()`, for machines driven by ticks instead of
    /// distinct events.
    ///
    /// Behaves exactly like [`handle`](InitializedStateMachine::handle) called
    /// with `()`.
    pub async fn step(&mut self, context: &mut M::Context<'_, '_>) -> Option<&M::Exit>
    where
        M: IntoStateMachine<Event = ()>,
    {
        self.handle((), context).await
    }

    /// Complete any interrupted work, then reset a terminated machine to
    /// [`IntoStateMachine::initial`], clear its exit value and await the
    /// initial entry actions again.
    ///
    /// Does nothing else on a running machine. The shared storage is kept.
    pub async fn restart(&mut self, context: &mut M::Context<'_, '_>) {
        self.inner.restart(context).await;
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

// MARK: Lazily Initialized Machine

/// An awaitable state machine that runs its initial entry actions on first use.
///
/// [`handle`](StateMachine::handle) and [`step`](StateMachine::step) initialize
/// an uninitialized machine before dispatching, which suits callers that have
/// no natural place for a separate initialization step. Prefer
/// [`UninitializedStateMachine`] and [`InitializedStateMachine`] when the type
/// should record that initialization has happened.
///
/// The machine terminates when an [`Outcome::Exit`](crate::Outcome::Exit)
/// reaches it. A terminated machine keeps its last state for inspection and
/// ignores events until [`restart`](StateMachine::restart) is awaited.
///
/// # Cancellation
///
/// The machine records its progress after every completed action and hook of
/// an initialization, a transition or a termination. If the future of
/// [`init`](StateMachine::init), [`handle`](StateMachine::handle),
/// [`step`](StateMachine::step) or [`restart`](StateMachine::restart) is
/// dropped while it awaits one of them, the next of these calls first
/// completes the interrupted work. The interrupted action or hook runs again
/// from its start, and every completed one is never repeated. An interrupted
/// initialization counts as begun, so it is completed and never started over.
/// On an initialized machine, `init` completes the interrupted work and does
/// nothing else, so a completed termination leaves the machine terminated.
///
/// Dropping the future while the event is being dispatched, that is while a
/// handler or a dispatch hook is awaited, loses the event: nothing is resumed
/// for it.
///
/// # Examples
///
/// ```
/// use core::convert::Infallible;
///
/// use futures::executor::block_on;
/// use sotu::awaitable::{IntoStateMachine, State, StateMachine};
/// use sotu::prelude::*;
///
/// #[derive(Default)]
/// struct Counter {
///     entries: u32,
///     events: u32,
/// }
///
/// enum CounterState {
///     Counting,
/// }
///
/// impl IntoStateMachine for Counter {
///     type Event = ();
///     type Context<'ctx, 'data: 'ctx> = ();
///     type State = CounterState;
///     type Superstate<'sub> = ();
///     type Exit = Infallible;
///
///     fn initial() -> CounterState {
///         CounterState::Counting
///     }
/// }
///
/// impl State<Counter> for CounterState {
///     async fn call_handler<'ctx, 'data: 'ctx>(
///         &mut self,
///         shared_storage: &mut Counter,
///         _event: &(),
///         _context: &mut (),
///     ) -> Outcome<CounterState, Infallible> {
///         shared_storage.events += 1;
///
///         Outcome::Handled
///     }
///
///     async fn call_entry_action<'ctx, 'data: 'ctx>(
///         &mut self,
///         shared_storage: &mut Counter,
///         _context: &mut (),
///     ) {
///         shared_storage.entries += 1;
///     }
/// }
///
/// block_on(async {
///     let mut machine: StateMachine<Counter> = Counter::default().state_machine();
///
///     assert_eq!(machine.entries, 0);
///
///     machine.handle((), &mut ()).await;
///
///     assert_eq!(machine.entries, 1);
///     assert_eq!(machine.events, 1);
///
///     machine.handle((), &mut ()).await;
///
///     assert_eq!(machine.entries, 1);
///     assert_eq!(machine.events, 2);
/// });
/// ```
pub struct StateMachine<M: IntoStateMachine> {
    inner: Inner<M>,
    /// Whether the initialization has begun, even if it was interrupted.
    initialized: bool,
}

impl<M: IntoStateMachine> StateMachine<M> {
    /// Await the entry actions of every level of the initial state, outermost
    /// first, unless the machine is already initialized.
    ///
    /// On an initialized machine this only completes any interrupted
    /// initialization, transition or termination, and never restarts a
    /// terminated machine. No transition hook fires for the initial entry,
    /// because no source state exists.
    pub async fn init(&mut self, context: &mut M::Context<'_, '_>) {
        if self.initialized {
            self.inner.drive_pending(context).await;

            return;
        }

        // The first poll of `Inner::init` records the initialization before it
        // first suspends, so a future dropped after that poll leaves a record
        // that the next call completes instead of starting a second
        // initialization.
        self.initialized = true;
        self.inner.init(context).await;
    }

    /// Initialize the machine if needed, complete any interrupted work, then
    /// dispatch `event` along the active path and perform the resulting
    /// transition or termination.
    ///
    /// Returns the exit value if this call's event terminated the machine,
    /// borrowed from the recorded value that [`exit`](StateMachine::exit) also
    /// returns. Returns `None` otherwise, in particular on a terminated
    /// machine, which does not dispatch the event at all, and when the call
    /// only completed an interrupted termination.
    pub async fn handle(
        &mut self,
        event: impl Into<M::Event>,
        context: &mut M::Context<'_, '_>,
    ) -> Option<&M::Exit> {
        self.init(context).await;

        self.inner.handle(event.into(), context).await
    }

    /// Dispatch the unit event `()`, for machines driven by ticks instead of
    /// distinct events.
    ///
    /// Behaves exactly like [`handle`](StateMachine::handle) called with `()`,
    /// including the initialization of an uninitialized machine.
    pub async fn step(&mut self, context: &mut M::Context<'_, '_>) -> Option<&M::Exit>
    where
        M: IntoStateMachine<Event = ()>,
    {
        self.handle((), context).await
    }

    /// Complete any interrupted work, then reset a terminated machine to
    /// [`IntoStateMachine::initial`], clear its exit value and await the
    /// initial entry actions again.
    ///
    /// Does nothing else on a running machine or on an uninitialized machine.
    /// The shared storage is kept.
    pub async fn restart(&mut self, context: &mut M::Context<'_, '_>) {
        // Only a machine that handled an event can have terminated, so
        // `Inner::restart` already skips an uninitialized machine.
        self.inner.restart(context).await;
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

impl<M: IntoStateMachine> Deref for StateMachine<M> {
    type Target = M;

    fn deref(&self) -> &M {
        &self.inner.shared_storage
    }
}

impl<M: IntoStateMachine> From<UninitializedStateMachine<M>> for StateMachine<M> {
    /// Convert into a machine that still initializes itself on first use.
    fn from(machine: UninitializedStateMachine<M>) -> Self {
        Self {
            inner: machine.inner,
            initialized: false,
        }
    }
}

impl<M: IntoStateMachine> From<InitializedStateMachine<M>> for StateMachine<M> {
    /// Convert into a machine that does not run the initial entry actions
    /// again.
    ///
    /// Work that a dropped future of the initialized machine left interrupted
    /// stays recorded and is completed by the next call.
    fn from(machine: InitializedStateMachine<M>) -> Self {
        Self {
            inner: machine.inner,
            initialized: true,
        }
    }
}

impl<M: IntoStateMachine + Default> Default for StateMachine<M> {
    /// Return an uninitialized machine holding the default shared storage.
    fn default() -> Self {
        M::default().state_machine()
    }
}
