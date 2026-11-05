/// The result of a state, superstate or mount handler.
///
/// `Handled` consumes the event. `Super` defers the event to the parent
/// node; a `Super` returned past the top-level nodes of the root machine
/// counts as handled. `Transition` performs a local transition to the given
/// target, while `ExternalTransition` performs an external transition to it.
/// `Exit` leaves the machine through the exit point `X`, which defaults to
/// [`Infallible`](core::convert::Infallible) for machines without exit
/// points.
///
/// # Examples
///
/// ```
/// use sotu::Outcome;
///
/// let outcome: Outcome<u8> = Outcome::Transition(3);
///
/// assert_eq!(outcome, Outcome::Transition(3));
/// assert_ne!(outcome, Outcome::ExternalTransition(3));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Outcome<S, X = core::convert::Infallible> {
    /// Consume the event.
    Handled,
    /// Defer the event to the parent node.
    Super,
    /// Transition locally to the target.
    Transition(S),
    /// Transition externally to the target.
    ExternalTransition(S),
    /// Leave the machine through the given exit point.
    Exit(X),
}
