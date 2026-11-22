#![cfg(feature = "async")]

mod common;

use common::async_tree::async_state_for;
use common::tree::Leaf;
use common::tree::depth_of;
use common::tree::leaf_node;
use common::tree::state_for;
use proptest::prop_assert_eq;
use sotu::awaitable::StateExt as _;
use sotu::blocking::StateExt as _;
use test_strategy::proptest;

// MARK: Depth

#[test]
fn depth_of_every_leaf() {
    assert_eq!(async_state_for(Leaf::A1).depth(), 2);
    assert_eq!(async_state_for(Leaf::B1).depth(), 3);
    assert_eq!(async_state_for(Leaf::B2).depth(), 3);
    assert_eq!(async_state_for(Leaf::C).depth(), 1);
}

// MARK: Transition Paths

// The blocking algorithms are the reference: both modes describe the same
// hierarchy, so they must agree on every path.

#[proptest]
fn transition_path_matches_blocking(source: Leaf, target: Leaf) {
    let mut async_source = async_state_for(source);
    let mut async_target = async_state_for(target);
    let mut blocking_source = state_for(source);
    let mut blocking_target = state_for(target);

    prop_assert_eq!(
        async_source.transition_path(&mut async_target),
        blocking_source.transition_path(&mut blocking_target)
    );
}

#[proptest]
fn external_transition_path_matches_blocking(
    source: Leaf,
    target: Leaf,
    #[strategy(0usize..3)] handler_level: usize,
) {
    // A level at or above the depth of `source` yields the handler depth `0`,
    // which names no handling node.
    let handler_depth = depth_of(leaf_node(source)).saturating_sub(handler_level);
    let mut async_source = async_state_for(source);
    let mut async_target = async_state_for(target);
    let mut blocking_source = state_for(source);
    let mut blocking_target = state_for(target);

    prop_assert_eq!(
        async_source.external_transition_path(&mut async_target, handler_depth),
        blocking_source.external_transition_path(&mut blocking_target, handler_depth)
    );
}
