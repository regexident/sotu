#![cfg(feature = "async")]

mod common;

use common::async_tree::{AsyncTree, AsyncTreeState};
use common::tree::{Finished, Input, Leaf, Node, Record};
use futures::executor::block_on;
use sotu::awaitable::{InitializedStateMachine, IntoStateMachineExt};

// MARK: Helpers

/// Return an initialized machine in `leaf` with an empty trace.
async fn machine_at(leaf: Leaf) -> InitializedStateMachine<AsyncTree> {
    let mut machine = AsyncTree::default()
        .uninitialized_state_machine()
        .init(&mut ())
        .await;

    if leaf != Leaf::A1 {
        machine.handle(Input::Go(leaf), &mut ()).await;
    }

    machine.inner_mut().trace.clear();

    machine
}

fn dispatch_records(node: Node) -> [Record; 3] {
    [
        Record::BeforeDispatch(node),
        Record::Handler(node),
        Record::AfterDispatch(node),
    ]
}

// MARK: Initialization

#[test]
fn init_enters_initial_state_outermost_first() {
    block_on(async {
        let uninitialized = AsyncTree::default().uninitialized_state_machine();

        assert_eq!(uninitialized.trace, []);
        assert_eq!(uninitialized.state(), &AsyncTreeState::A1);

        let machine = uninitialized.init(&mut ()).await;

        assert_eq!(
            machine.trace,
            [Record::Entry(Node::A), Record::Entry(Node::A1)],
        );
        assert_eq!(machine.state(), &AsyncTreeState::A1);
    });
}

// MARK: Transitions

#[test]
fn sibling_transition_runs_hooks_and_actions_in_order() {
    block_on(async {
        let mut machine = machine_at(Leaf::A1).await;

        assert_eq!(machine.handle(Input::Go(Leaf::B1), &mut ()).await, None);
        assert_eq!(
            machine.trace,
            [
                Record::BeforeDispatch(Node::A1),
                Record::Handler(Node::A1),
                Record::AfterDispatch(Node::A1),
                Record::BeforeTransition(Leaf::A1, Leaf::B1),
                Record::Exit(Node::A1),
                Record::Entry(Node::B),
                Record::Entry(Node::B1),
                Record::AfterTransition(Leaf::A1, Leaf::B1),
            ],
        );
        assert_eq!(machine.state(), &AsyncTreeState::B1 { visits: 2 });
    });
}

#[test]
fn external_transition_reenters_handling_superstate() {
    block_on(async {
        let mut machine = machine_at(Leaf::B1).await;

        assert_eq!(
            machine
                .handle(Input::ExternalFromB(Leaf::B2), &mut ())
                .await,
            None
        );
        assert_eq!(
            machine.trace,
            [
                dispatch_records(Node::B1).as_slice(),
                dispatch_records(Node::B).as_slice(),
                &[
                    Record::BeforeTransition(Leaf::B1, Leaf::B2),
                    Record::Exit(Node::B1),
                    Record::Exit(Node::B),
                    Record::Entry(Node::B),
                    Record::Entry(Node::B2),
                    Record::AfterTransition(Leaf::B1, Leaf::B2),
                ],
            ]
            .concat(),
        );
        assert_eq!(machine.state(), &AsyncTreeState::B2 { visits: 2 });
    });
}

// MARK: Termination

#[test]
fn termination_runs_every_exit_action_and_reports_exit() {
    block_on(async {
        let mut machine = machine_at(Leaf::B1).await;

        assert_eq!(
            machine.handle(Input::Finish, &mut ()).await,
            Some(&Finished::Done)
        );
        assert_eq!(
            machine.trace,
            [
                Record::BeforeDispatch(Node::B1),
                Record::Handler(Node::B1),
                Record::AfterDispatch(Node::B1),
                Record::Exit(Node::B1),
                Record::Exit(Node::B),
                Record::Exit(Node::A),
            ],
        );
        assert!(machine.is_terminated());
        assert_eq!(machine.exit(), Some(&Finished::Done));
    });
}

#[test]
fn terminated_machine_ignores_events() {
    block_on(async {
        let mut machine = machine_at(Leaf::C).await;

        machine.handle(Input::Finish, &mut ()).await;

        let trace_at_termination = machine.trace.clone();

        assert_eq!(machine.handle(Input::Go(Leaf::A1), &mut ()).await, None);
        assert_eq!(machine.trace, trace_at_termination);
        assert_eq!(machine.state(), &AsyncTreeState::C);
        assert_eq!(machine.exit(), Some(&Finished::Done));
    });
}
