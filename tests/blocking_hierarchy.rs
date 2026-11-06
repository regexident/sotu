mod common;

use std::iter::successors;

use common::tree::{Leaf, Node, depth_of, leaf_node, parent_of, state_for};
use proptest::prop_assert_eq;
use sotu::blocking::StateExt;
use test_strategy::proptest;

// MARK: Reference

// The reference is built only on `parent_of` and `depth_of`, so it shares no
// code with the algorithms under test.

fn reference_ancestors(node: Node) -> Vec<Node> {
    successors(parent_of(node), |ancestor| parent_of(*ancestor)).collect()
}

fn reference_transition_path(source: Leaf, target: Leaf) -> (usize, usize) {
    if source == target {
        return (1, 1);
    }

    let source_node = leaf_node(source);
    let target_node = leaf_node(target);
    let target_ancestors = reference_ancestors(target_node);
    let common_depth = reference_ancestors(source_node)
        .into_iter()
        .find(|ancestor| target_ancestors.contains(ancestor))
        .map_or(0, depth_of);

    (
        depth_of(source_node) - common_depth,
        depth_of(target_node) - common_depth,
    )
}

// MARK: Depth

#[test]
fn depth_of_every_leaf() {
    assert_eq!(state_for(Leaf::A1).depth(), 2);
    assert_eq!(state_for(Leaf::B1).depth(), 3);
    assert_eq!(state_for(Leaf::B2).depth(), 3);
    assert_eq!(state_for(Leaf::C).depth(), 1);
}

// MARK: Transition Paths

#[proptest]
fn transition_path_matches_reference(source: Leaf, target: Leaf) {
    let mut source_state = state_for(source);
    let mut target_state = state_for(target);

    prop_assert_eq!(
        source_state.transition_path(&mut target_state),
        reference_transition_path(source, target)
    );
}
