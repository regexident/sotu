use crate::StateOrSuperstate;

use super::{State, Superstate};

/// Describe a state machine whose shared storage is `Self`.
///
/// The implementing type holds the storage that every state and superstate can
/// access mutably while handling an event. The associated types name the
/// machine's event, context, leaf states, superstates and exit points.
///
/// Superstates are built on demand from the active leaf state through
/// [`State::superstate`], so [`Self::Superstate`] may borrow the leaf's local
/// storage for the lifetime `'sub`.
///
/// Contexts take two lifetimes because `&'a mut T` is invariant in `T`: with a
/// single lifetime, a context such as `&'a mut Writer<'a>` could not be
/// rebuilt for each event inside a loop.
///
/// The four hooks observe dispatch and transitions without influencing them.
/// Each defaults to doing nothing.
///
/// [`Self::Superstate`]: IntoStateMachine::Superstate
pub trait IntoStateMachine: Sized {
    /// The event type handled by the machine.
    type Event;

    /// The context handed to every handler, entry action and exit action.
    type Context<'ctx, 'data: 'ctx>;

    /// The leaf states of the machine.
    type State: State<Self>;

    /// The superstates of the machine, borrowing local storage from the
    /// active leaf state for `'sub`.
    ///
    /// The bound `Self: 'sub` covers every parameter the state type can
    /// mention, because every parameter of an implementation appears in `Self`.
    type Superstate<'sub>: Superstate<Self>
    where
        Self: 'sub;

    /// The exit points through which the machine can be left.
    type Exit;

    /// Return the state the machine starts in.
    fn initial() -> Self::State;

    /// Observe `event` just before `node` handles it.
    fn before_dispatch<'ctx, 'data: 'ctx>(
        &mut self,
        _node: StateOrSuperstate<'_, Self::State, Self::Superstate<'_>>,
        _event: &Self::Event,
        _context: &mut Self::Context<'ctx, 'data>,
    ) {
    }

    /// Observe `event` just after `node` handled it.
    fn after_dispatch<'ctx, 'data: 'ctx>(
        &mut self,
        _node: StateOrSuperstate<'_, Self::State, Self::Superstate<'_>>,
        _event: &Self::Event,
        _context: &mut Self::Context<'ctx, 'data>,
    ) {
    }

    /// Observe a transition from `source` to `target` just before any exit
    /// action runs.
    fn before_transition<'ctx, 'data: 'ctx>(
        &mut self,
        _source: &Self::State,
        _target: &Self::State,
        _context: &mut Self::Context<'ctx, 'data>,
    ) {
    }

    /// Observe a transition from `source` to `target` just after the last
    /// entry action ran.
    fn after_transition<'ctx, 'data: 'ctx>(
        &mut self,
        _source: &Self::State,
        _target: &Self::State,
        _context: &mut Self::Context<'ctx, 'data>,
    ) {
    }
}
