/// Identifies the node a dispatch hook is invoked for.
///
/// # Examples
///
/// ```
/// use sotu::StateOrSuperstate;
///
/// let superstate = 7u8;
/// let node: StateOrSuperstate<'_, u16, u8> = (&superstate).into();
///
/// assert_eq!(node, StateOrSuperstate::Superstate(&7u8));
/// ```
#[derive(Debug, PartialEq, Eq)]
pub enum StateOrSuperstate<'a, State, Superstate> {
    /// A leaf state.
    State(&'a State),
    /// A superstate.
    Superstate(&'a Superstate),
}

impl<'a, State, Superstate> From<&'a Superstate> for StateOrSuperstate<'a, State, Superstate> {
    fn from(superstate: &'a Superstate) -> Self {
        Self::Superstate(superstate)
    }
}

/// Represents a mounted child machine inside the parent's superstate type.
///
/// # Examples
///
/// ```
/// use sotu::SubmachineNode;
///
/// assert_ne!(SubmachineNode::<u8>::Mount, SubmachineNode::Inner(3));
/// ```
#[derive(Debug, PartialEq, Eq)]
pub enum SubmachineNode<S> {
    /// The mount node itself.
    Mount,
    /// A superstate of the child machine.
    Inner(S),
}
