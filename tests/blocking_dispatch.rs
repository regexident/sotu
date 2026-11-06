mod common;

use core::convert::Infallible;

use common::tree::{
    Finished, Input, Leaf, Node, Record, Tree, TreeState, TreeSuperstate, state_for,
};
use sotu::StateOrSuperstate;
use sotu::blocking::{IntoStateMachine, State, StateExt, Superstate};
use sotu::prelude::Outcome;
use sotu::private::{Dispatched, NodeOutcome};

// MARK: Chain Machine

// A chain `Top > Middle > Leaf` whose leaf produces the signals that mounts
// emit, which the handler-based `Tree` machine cannot express.

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum ChainNode {
    Leaf,
    Middle,
    Top,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum ChainRecord {
    BeforeDispatch(ChainNode),
    Handler(ChainNode),
    AfterDispatch(ChainNode),
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum ChainInput {
    Unaddressed,
    Defer(usize),
    External(usize),
}

#[derive(Default)]
struct Chain {
    trace: Vec<ChainRecord>,
}

#[derive(Eq, PartialEq, Debug)]
enum ChainState {
    Leaf,
}

enum ChainSuperstate {
    Middle,
    Top,
}

fn chain_node_of(node: StateOrSuperstate<'_, ChainState, ChainSuperstate>) -> ChainNode {
    match node {
        StateOrSuperstate::State(ChainState::Leaf) => ChainNode::Leaf,
        StateOrSuperstate::Superstate(ChainSuperstate::Middle) => ChainNode::Middle,
        StateOrSuperstate::Superstate(ChainSuperstate::Top) => ChainNode::Top,
    }
}

impl IntoStateMachine for Chain {
    type Event = ChainInput;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = ChainState;
    type Superstate<'sub> = ChainSuperstate;
    type Exit = Infallible;

    fn initial() -> ChainState {
        ChainState::Leaf
    }

    fn before_dispatch<'ctx, 'data: 'ctx>(
        &mut self,
        node: StateOrSuperstate<'_, ChainState, ChainSuperstate>,
        _event: &ChainInput,
        _context: &mut (),
    ) {
        self.trace
            .push(ChainRecord::BeforeDispatch(chain_node_of(node)));
    }

    fn after_dispatch<'ctx, 'data: 'ctx>(
        &mut self,
        node: StateOrSuperstate<'_, ChainState, ChainSuperstate>,
        _event: &ChainInput,
        _context: &mut (),
    ) {
        self.trace
            .push(ChainRecord::AfterDispatch(chain_node_of(node)));
    }
}

impl State<Chain> for ChainState {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Chain,
        _event: &ChainInput,
        _context: &mut (),
    ) -> Outcome<ChainState, Infallible> {
        Outcome::Super
    }

    fn superstate(&mut self) -> Option<ChainSuperstate> {
        Some(ChainSuperstate::Middle)
    }

    fn __is_addressed(&self, event: ChainInput) -> (bool, ChainInput) {
        (event != ChainInput::Unaddressed, event)
    }

    fn __dispatch<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Chain,
        event: ChainInput,
        _context: &mut (),
    ) -> Dispatched<ChainState, Infallible, ChainInput> {
        shared_storage
            .trace
            .push(ChainRecord::Handler(ChainNode::Leaf));

        let outcome = match event {
            ChainInput::Unaddressed => NodeOutcome::Super,
            ChainInput::Defer(mount_offset) => NodeOutcome::DeferToMount { mount_offset },
            ChainInput::External(handler_offset) => NodeOutcome::ExternalTransition {
                target: ChainState::Leaf,
                handler_offset,
            },
        };

        Dispatched { outcome, event }
    }
}

impl Superstate<Chain> for ChainSuperstate {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Chain,
        _event: &ChainInput,
        _context: &mut (),
    ) -> Outcome<ChainState, Infallible> {
        match self {
            ChainSuperstate::Middle => {
                shared_storage
                    .trace
                    .push(ChainRecord::Handler(ChainNode::Middle));

                Outcome::Super
            }
            ChainSuperstate::Top => {
                shared_storage
                    .trace
                    .push(ChainRecord::Handler(ChainNode::Top));

                Outcome::Handled
            }
        }
    }

    fn superstate(&mut self) -> Option<ChainSuperstate> {
        match self {
            ChainSuperstate::Middle => Some(ChainSuperstate::Top),
            ChainSuperstate::Top => None,
        }
    }

    fn same_state(&self, other: &ChainSuperstate) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

// MARK: Helpers

fn run_tree(leaf: Leaf, input: Input) -> ((Outcome<TreeState, Finished>, usize), Vec<Record>) {
    let mut tree = Tree::default();
    let result = state_for(leaf).handle(&mut tree, input, &mut ());

    (result, tree.trace)
}

fn run_chain(input: ChainInput) -> ((Outcome<ChainState, Infallible>, usize), Vec<ChainRecord>) {
    let mut chain = Chain::default();
    let result = ChainState::Leaf.handle(&mut chain, input, &mut ());

    (result, chain.trace)
}

fn tree_node_records(node: Node) -> [Record; 3] {
    [
        Record::BeforeDispatch(node),
        Record::Handler(node),
        Record::AfterDispatch(node),
    ]
}

fn chain_node_records(node: ChainNode) -> [ChainRecord; 3] {
    [
        ChainRecord::BeforeDispatch(node),
        ChainRecord::Handler(node),
        ChainRecord::AfterDispatch(node),
    ]
}

// MARK: Tree Tests

#[test]
fn event_bubbles_to_top_with_hooks_around_each_node() {
    let (result, trace) = run_tree(Leaf::B1, Input::Ignore);

    assert_eq!(result, (Outcome::Handled, 0));
    assert_eq!(
        trace,
        [
            tree_node_records(Node::B1),
            tree_node_records(Node::B),
            tree_node_records(Node::A),
        ]
        .concat(),
    );
}

#[test]
fn first_non_super_outcome_wins() {
    let (result, trace) = run_tree(Leaf::B1, Input::GoFromA(Leaf::C));

    assert_eq!(result, (Outcome::Transition(TreeState::C), 1));
    assert_eq!(
        trace,
        [
            tree_node_records(Node::B1),
            tree_node_records(Node::B),
            tree_node_records(Node::A),
        ]
        .concat(),
    );

    // Starting at the superstate `B` visits only `B` and `A`, and the depth
    // still counts from the top rather than from the starting node.
    let mut tree = Tree::default();
    let mut visits = 0;
    let mut superstate = TreeSuperstate::B {
        visits: &mut visits,
    };
    let result = sotu::blocking::SuperstateExt::handle(
        &mut superstate,
        &mut tree,
        Input::GoFromA(Leaf::C),
        &mut (),
    );

    assert_eq!(result, (Outcome::Transition(TreeState::C), 1));
    assert_eq!(
        tree.trace,
        [tree_node_records(Node::B), tree_node_records(Node::A)].concat(),
    );
}

#[test]
fn external_transition_reports_handler_depth() {
    let (result, trace) = run_tree(Leaf::B1, Input::ExternalFromB(Leaf::B2));

    assert_eq!(
        result,
        (Outcome::ExternalTransition(TreeState::B2 { visits: 0 }), 2),
    );
    assert_eq!(
        trace,
        [tree_node_records(Node::B1), tree_node_records(Node::B)].concat(),
    );
}

#[test]
fn leaf_exit_reports_leaf_depth() {
    let (result, trace) = run_tree(Leaf::C, Input::Finish);

    assert_eq!(result, (Outcome::Exit(Finished::Done), 1));
    assert_eq!(trace, tree_node_records(Node::C));
}

// MARK: Chain Tests

#[test]
fn unaddressed_nodes_are_skipped_without_hooks() {
    let (result, trace) = run_chain(ChainInput::Unaddressed);

    assert_eq!(result, (Outcome::Handled, 1));
    assert_eq!(
        trace,
        [
            chain_node_records(ChainNode::Middle),
            chain_node_records(ChainNode::Top),
        ]
        .concat(),
    );
}

#[test]
fn defer_to_mount_skips_intermediate_nodes() {
    let (result, trace) = run_chain(ChainInput::Defer(2));

    assert_eq!(result, (Outcome::Handled, 1));
    assert_eq!(
        trace,
        [
            chain_node_records(ChainNode::Leaf),
            chain_node_records(ChainNode::Top),
        ]
        .concat(),
    );
}

#[test]
fn handler_offset_moves_handling_depth() {
    let (result, trace) = run_chain(ChainInput::External(1));

    assert_eq!(result, (Outcome::ExternalTransition(ChainState::Leaf), 2));
    assert_eq!(trace, chain_node_records(ChainNode::Leaf));
}

#[test]
fn oversized_defer_offset_drops_event() {
    let (result, trace) = run_chain(ChainInput::Defer(3));

    assert_eq!(result, (Outcome::Handled, 0));
    assert_eq!(trace, chain_node_records(ChainNode::Leaf));
}
