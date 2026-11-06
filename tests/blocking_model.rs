mod common;

use std::iter::successors;

use common::tree::{Input, Leaf, Node, Record, Tree, depth_of, leaf_node, leaf_of, parent_of};
use proptest::prelude::{BoxedStrategy, Just, Strategy, any};
use proptest_state_machine::{ReferenceStateMachine, StateMachineTest, prop_state_machine};
use sotu::blocking::{IntoStateMachineExt, StateMachine};
use test_strategy::Arbitrary;

// MARK: Reference Model

// The model computes every path from `parent_of` and `depth_of` and shares no
// code with the runtime, so an error in the runtime's algorithms cannot hide by
// also appearing here.

#[derive(Clone, Debug)]
struct TreeReference {
    active: Leaf,
    terminated: bool,
}

#[derive(Clone, Debug, Arbitrary)]
enum Operation {
    Handle(Input),
    Restart,
}

#[derive(Copy, Clone)]
enum ReferenceOutcome {
    Defer,
    Local(Leaf),
    External(Leaf),
    Terminate,
}

/// Return what the handler of `node` decides for `input`.
fn reference_outcome(node: Node, input: Input) -> ReferenceOutcome {
    match (node, input) {
        (Node::A, Input::GoFromA(target)) => ReferenceOutcome::Local(target),
        (Node::A, Input::ExternalFromA(target)) | (Node::B, Input::ExternalFromB(target)) => {
            ReferenceOutcome::External(target)
        }
        (Node::A | Node::B, _) => ReferenceOutcome::Defer,
        (_, Input::Go(target)) => ReferenceOutcome::Local(target),
        (_, Input::ExternalFromLeaf(target)) => ReferenceOutcome::External(target),
        (Node::B1 | Node::C, Input::Finish) => ReferenceOutcome::Terminate,
        _ => ReferenceOutcome::Defer,
    }
}

/// Return the active path of `leaf`, from the leaf up to its top-level node.
fn active_path(leaf: Leaf) -> Vec<Node> {
    successors(Some(leaf_node(leaf)), |node| parent_of(*node)).collect()
}

/// Return the nodes of `path` that `other_path` does not contain.
fn nodes_outside(path: &[Node], other_path: &[Node]) -> Vec<Node> {
    path.iter()
        .copied()
        .filter(|node| !other_path.contains(node))
        .collect()
}

/// Return the nodes a local transition exits and enters, both innermost first.
///
/// A transition to the same leaf exits and enters exactly that leaf. Any other
/// transition leaves the common ancestors of the two leaves untouched.
fn local_path(source: Leaf, target: Leaf) -> (Vec<Node>, Vec<Node>) {
    if source == target {
        return (vec![leaf_node(source)], vec![leaf_node(target)]);
    }

    let source_path = active_path(source);
    let target_path = active_path(target);

    (
        nodes_outside(&source_path, &target_path),
        nodes_outside(&target_path, &source_path),
    )
}

/// Return the nodes an external transition handled by `handler` exits and
/// enters, both innermost first.
///
/// The handler itself is left and entered again when it is the source leaf
/// transitioning to itself or a superstate enclosing both leaves. Every other
/// external transition takes the local path.
fn external_path(source: Leaf, target: Leaf, handler: Node) -> (Vec<Node>, Vec<Node>) {
    let handler_is_reentered = if handler == leaf_node(source) {
        source == target
    } else {
        active_path(target).contains(&handler)
    };

    if !handler_is_reentered {
        return local_path(source, target);
    }

    let at_or_below_handler = |leaf| {
        active_path(leaf)
            .into_iter()
            .filter(|node| depth_of(*node) >= depth_of(handler))
            .collect()
    };

    (at_or_below_handler(source), at_or_below_handler(target))
}

/// Return the records of a transition: the exit actions innermost first and the
/// entry actions outermost first, surrounded by the transition hooks.
fn transition_records(
    source: Leaf,
    target: Leaf,
    (exited, entered): (Vec<Node>, Vec<Node>),
) -> Vec<Record> {
    let mut records = vec![Record::BeforeTransition(source, target)];

    records.extend(exited.into_iter().map(Record::Exit));
    records.extend(entered.into_iter().rev().map(Record::Entry));
    records.push(Record::AfterTransition(source, target));

    records
}

/// Return the records and next state of dispatching `input` on a running
/// machine.
///
/// The event climbs the active path until a node decides. An event every node
/// defers is dropped.
fn expected_dispatch(active: Leaf, input: Input) -> (Vec<Record>, Leaf, bool) {
    let mut records = Vec::new();

    for node in active_path(active) {
        records.push(Record::BeforeDispatch(node));
        records.push(Record::Handler(node));
        records.push(Record::AfterDispatch(node));

        match reference_outcome(node, input) {
            ReferenceOutcome::Defer => {}
            ReferenceOutcome::Local(target) => {
                let path = local_path(active, target);

                records.extend(transition_records(active, target, path));

                return (records, target, false);
            }
            ReferenceOutcome::External(target) => {
                let path = external_path(active, target, node);

                records.extend(transition_records(active, target, path));

                return (records, target, false);
            }
            ReferenceOutcome::Terminate => {
                records.extend(active_path(active).into_iter().map(Record::Exit));

                return (records, active, true);
            }
        }
    }

    (records, active, false)
}

/// Return the exact records one step appends to the trace, the next active leaf
/// and the next termination flag.
fn expected_step(
    active: Leaf,
    terminated: bool,
    operation: &Operation,
) -> (Vec<Record>, Leaf, bool) {
    match (operation, terminated) {
        (Operation::Handle(input), false) => expected_dispatch(active, *input),
        (Operation::Handle(_), true) => (Vec::new(), active, true),
        (Operation::Restart, false) => (Vec::new(), active, false),
        (Operation::Restart, true) => {
            let records = active_path(Leaf::A1)
                .into_iter()
                .rev()
                .map(Record::Entry)
                .collect();

            (records, Leaf::A1, false)
        }
    }
}

impl ReferenceStateMachine for TreeReference {
    type State = TreeReference;
    type Transition = Operation;

    fn init_state() -> BoxedStrategy<TreeReference> {
        Just(TreeReference {
            active: Leaf::A1,
            terminated: false,
        })
        .boxed()
    }

    fn transitions(_state: &TreeReference) -> BoxedStrategy<Operation> {
        any::<Operation>().boxed()
    }

    fn apply(state: TreeReference, transition: &Operation) -> TreeReference {
        let (_, active, terminated) = expected_step(state.active, state.terminated, transition);

        TreeReference { active, terminated }
    }
}

// MARK: System Under Test

struct TreeModelTest;

impl StateMachineTest for TreeModelTest {
    type SystemUnderTest = StateMachine<Tree>;
    type Reference = TreeReference;

    fn init_test(_reference: &TreeReference) -> StateMachine<Tree> {
        let mut machine = Tree::default().state_machine();

        machine.init(&mut ());
        machine.inner_mut().trace.clear();

        machine
    }

    fn apply(
        mut machine: StateMachine<Tree>,
        _reference: &TreeReference,
        transition: Operation,
    ) -> StateMachine<Tree> {
        let active = leaf_of(machine.state());
        let terminated = machine.is_terminated();
        let previous_length = machine.trace.len();

        match transition {
            Operation::Handle(input) => {
                machine.handle(input, &mut ());
            }
            Operation::Restart => {
                machine.restart(&mut ());
            }
        }

        let appended_records = machine.trace[previous_length..].to_vec();
        let (expected_records, _, _) = expected_step(active, terminated, &transition);

        assert_eq!(appended_records, expected_records);

        machine
    }

    fn check_invariants(machine: &StateMachine<Tree>, reference: &TreeReference) {
        assert_eq!(leaf_of(machine.state()), reference.active);
        assert_eq!(machine.is_terminated(), reference.terminated);
    }
}

// MARK: Tests

prop_state_machine! {
    #[test]
    fn tree_matches_reference_model(sequential 1..40 => TreeModelTest);
}
