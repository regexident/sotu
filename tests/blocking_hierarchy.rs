mod common;

use std::iter::successors;

use common::tree::{
    Leaf, Node, Record, Tree, TreeState, depth_of, leaf_node, parent_of, state_for,
};
use proptest::{prop_assert_eq, prop_assume};
use sotu::blocking::StateExt;
use test_strategy::proptest;

// MARK: Reference

// The reference is built only on `parent_of` and `depth_of`, so it shares no
// code with the algorithms under test.

fn reference_ancestors(node: Node) -> Vec<Node> {
    successors(parent_of(node), |ancestor| parent_of(*ancestor)).collect()
}

fn reference_ancestor_at_depth(node: Node, depth: usize) -> Node {
    successors(Some(node), |ancestor| parent_of(*ancestor))
        .find(|ancestor| depth_of(*ancestor) == depth)
        .expect("every depth up to the node's own has an ancestor")
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

fn reference_external_transition_path(
    source: Leaf,
    target: Leaf,
    handler_level: usize,
) -> (usize, usize) {
    let source_node = leaf_node(source);
    let target_node = leaf_node(target);
    let handler_depth = depth_of(source_node) - handler_level;
    let handler = reference_ancestor_at_depth(source_node, handler_depth);

    let handler_is_reentered = if handler_level == 0 {
        source == target
    } else {
        depth_of(target_node) > handler_depth
            && reference_ancestor_at_depth(target_node, handler_depth) == handler
    };

    if handler_is_reentered {
        return (
            depth_of(source_node) - handler_depth + 1,
            depth_of(target_node) - handler_depth + 1,
        );
    }

    reference_transition_path(source, target)
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

#[proptest]
fn external_transition_path_matches_reference(
    source: Leaf,
    target: Leaf,
    #[strategy(0usize..3)] handler_level: usize,
) {
    prop_assume!(handler_level < depth_of(leaf_node(source)));

    let handler_depth = depth_of(leaf_node(source)) - handler_level;
    let mut source_state = state_for(source);
    let mut target_state = state_for(target);

    prop_assert_eq!(
        source_state.external_transition_path(&mut target_state, handler_depth),
        reference_external_transition_path(source, target, handler_level)
    );
}

// MARK: Level-wise Actions

#[test]
fn enter_runs_entry_actions_outermost_first() {
    let mut tree = Tree::default();
    let mut state = state_for(Leaf::B1);

    state.enter(&mut tree, &mut (), 3);

    assert_eq!(
        tree.trace,
        vec![
            Record::Entry(Node::A),
            Record::Entry(Node::B),
            Record::Entry(Node::B1),
        ]
    );
    assert_eq!(state, TreeState::B1 { visits: 2 });
}

#[test]
fn exit_runs_exit_actions_innermost_first() {
    let mut tree = Tree::default();
    let mut state = TreeState::B2 { visits: 0 };

    state.exit(&mut tree, &mut (), 3);

    assert_eq!(
        tree.trace,
        vec![
            Record::Exit(Node::B2),
            Record::Exit(Node::B),
            Record::Exit(Node::A),
        ]
    );
    assert_eq!(state, TreeState::B2 { visits: 20 });
}

#[test]
fn levels_beyond_depth_do_nothing() {
    let mut tree = Tree::default();

    TreeState::C.enter_level(&mut tree, &mut (), 1);
    TreeState::C.exit_level(&mut tree, &mut (), 1);

    assert!(tree.trace.is_empty());
}
