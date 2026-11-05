use sotu::{Outcome, StateOrSuperstate, SubmachineNode};
use test_strategy::proptest;

#[test]
fn state_or_superstate_from_superstate_reference() {
    let superstate = 7u8;
    let node: StateOrSuperstate<'_, u16, u8> = (&superstate).into();

    assert_eq!(node, StateOrSuperstate::Superstate(&7u8));
    assert_ne!(node, StateOrSuperstate::State(&7u16));
}

#[proptest]
fn outcome_equality_follows_payload(first: u8, second: u8) {
    proptest::prop_assert_eq!(
        Outcome::<u8, u8>::Transition(first) == Outcome::Transition(second),
        first == second
    );
    proptest::prop_assert_eq!(
        Outcome::<u8, u8>::ExternalTransition(first) == Outcome::ExternalTransition(second),
        first == second
    );
    proptest::prop_assert_eq!(
        Outcome::<u8, u8>::Exit(first) == Outcome::Exit(second),
        first == second
    );
    proptest::prop_assert_ne!(
        Outcome::<u8, u8>::Transition(first),
        Outcome::ExternalTransition(first)
    );
}

#[test]
fn outcome_is_copy_and_hashable() {
    let outcome: Outcome<u8> = Outcome::Transition(3);
    let copied = outcome;

    assert_eq!(outcome, copied);

    let set: std::collections::HashSet<_> = [
        Outcome::<u8>::Handled,
        Outcome::Super,
        Outcome::Transition(1),
        Outcome::Transition(1),
    ]
    .into_iter()
    .collect();

    assert_eq!(set.len(), 3);
}

#[test]
fn submachine_node_equality() {
    assert_eq!(SubmachineNode::<u8>::Inner(3), SubmachineNode::Inner(3));
    assert_ne!(SubmachineNode::<u8>::Mount, SubmachineNode::Inner(3));
    assert_eq!(SubmachineNode::<u8>::Mount, SubmachineNode::Mount);
}

#[test]
fn prelude_exposes_outcome_variants() {
    use sotu::prelude::*;

    let outcomes: [Outcome<u8>; 4] = [Handled, Super, Transition(1), ExternalTransition(2)];

    assert!(matches!(outcomes[2], Transition(1)));
}
