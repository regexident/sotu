#![cfg(feature = "async")]

mod common;

use common::async_tree::AsyncTree;
use common::async_tree::AsyncTreeState;
use common::async_tree::async_state_for;
use common::tree::Finished;
use common::tree::Input;
use common::tree::Leaf;
use common::tree::Node;
use common::tree::Record;
use common::tree::Tree;
use common::tree::depth_of;
use common::tree::leaf_node;
use common::tree::state_for;
use futures::executor::block_on;
use proptest::prop_assert_eq;
use sotu::Outcome;
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

// MARK: Level-wise Actions

#[test]
fn enter_awaits_entry_actions_outermost_first() {
    let mut tree = AsyncTree::default();
    let mut state = async_state_for(Leaf::B1);

    block_on(state.enter(&mut tree, &mut (), 3));

    assert_eq!(
        tree.trace,
        vec![
            Record::Entry(Node::A),
            Record::Entry(Node::B),
            Record::Entry(Node::B1),
        ]
    );
    assert_eq!(state, AsyncTreeState::B1 { visits: 2 });
}

#[test]
fn exit_awaits_exit_actions_innermost_first() {
    let mut tree = AsyncTree::default();
    let mut state = async_state_for(Leaf::B1);

    block_on(state.exit(&mut tree, &mut (), 3));

    assert_eq!(
        tree.trace,
        vec![
            Record::Exit(Node::B1),
            Record::Exit(Node::B),
            Record::Exit(Node::A),
        ]
    );
    assert_eq!(state, AsyncTreeState::B1 { visits: 20 });
}

// MARK: Dispatch

fn run_async_tree(
    leaf: Leaf,
    input: Input,
) -> ((Outcome<AsyncTreeState, Finished>, usize), Vec<Record>) {
    let mut tree = AsyncTree::default();
    let result = block_on(async_state_for(leaf).handle(&mut tree, input, &mut ()));

    (result, tree.trace)
}

fn blocking_trace(leaf: Leaf, input: Input) -> Vec<Record> {
    let mut tree = Tree::default();
    state_for(leaf).handle(&mut tree, input, &mut ());

    tree.trace
}

fn node_records(node: Node) -> [Record; 3] {
    [
        Record::BeforeDispatch(node),
        Record::Handler(node),
        Record::AfterDispatch(node),
    ]
}

#[test]
fn event_bubbles_to_top_with_awaited_hooks() {
    let (result, trace) = run_async_tree(Leaf::B1, Input::Ignore);

    assert_eq!(result, (Outcome::Handled, 0));
    assert_eq!(
        trace,
        [
            node_records(Node::B1),
            node_records(Node::B),
            node_records(Node::A),
        ]
        .concat(),
    );
    assert_eq!(trace, blocking_trace(Leaf::B1, Input::Ignore));
}

#[test]
fn first_non_super_outcome_wins() {
    let (result, trace) = run_async_tree(Leaf::B1, Input::GoFromA(Leaf::C));

    assert_eq!(result, (Outcome::Transition(AsyncTreeState::C), 1));
    assert_eq!(
        trace,
        [
            node_records(Node::B1),
            node_records(Node::B),
            node_records(Node::A),
        ]
        .concat(),
    );
    assert_eq!(trace, blocking_trace(Leaf::B1, Input::GoFromA(Leaf::C)));
}
