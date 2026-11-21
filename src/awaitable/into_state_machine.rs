use super::{State, Superstate};

/// Describe an awaitable state machine whose shared storage is `Self`.
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
}
