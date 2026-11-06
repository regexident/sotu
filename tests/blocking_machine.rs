mod common;

use core::convert::Infallible;

use common::tree::{Finished, Input, Leaf, Node, Record, Tree, TreeState};
use sotu::blocking::{InitializedStateMachine, IntoStateMachine, IntoStateMachineExt, State};
use sotu::prelude::Outcome;

// MARK: Nested Machine

// A single-leaf machine reporting two levels of internal-transition nesting,
// which only mounts produce and the `Tree` machine cannot express.

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum NestedRecord {
    RootBefore,
    InternalBefore(usize),
    Exit,
    Entry,
    InternalAfter(usize),
    RootAfter,
}

#[derive(Default)]
struct Nested {
    trace: Vec<NestedRecord>,
}

enum NestedState {
    Only,
}

impl IntoStateMachine for Nested {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = NestedState;
    type Superstate<'sub> = ();
    type Exit = Infallible;

    fn initial() -> NestedState {
        NestedState::Only
    }

    fn before_transition<'ctx, 'data: 'ctx>(
        &mut self,
        _source: &NestedState,
        _target: &NestedState,
        _context: &mut (),
    ) {
        self.trace.push(NestedRecord::RootBefore);
    }

    fn after_transition<'ctx, 'data: 'ctx>(
        &mut self,
        _source: &NestedState,
        _target: &NestedState,
        _context: &mut (),
    ) {
        self.trace.push(NestedRecord::RootAfter);
    }
}

impl State<Nested> for NestedState {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Nested,
        _event: &(),
        _context: &mut (),
    ) -> Outcome<NestedState, Infallible> {
        Outcome::Transition(NestedState::Only)
    }

    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Nested,
        _context: &mut (),
    ) {
        shared_storage.trace.push(NestedRecord::Entry);
    }

    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Nested,
        _context: &mut (),
    ) {
        shared_storage.trace.push(NestedRecord::Exit);
    }

    fn __internal_transition_nesting(&mut self, _target: &mut Self, _exit_levels: usize) -> usize {
        2
    }

    fn __before_internal_transition<'ctx, 'data: 'ctx>(
        &mut self,
        _target: &mut Self,
        _exit_levels: usize,
        nesting: usize,
        shared_storage: &mut Nested,
        _context: &mut (),
    ) {
        shared_storage
            .trace
            .push(NestedRecord::InternalBefore(nesting));
    }

    fn __after_internal_transition<'ctx, 'data: 'ctx>(
        &mut self,
        _target: &mut Self,
        _exit_levels: usize,
        nesting: usize,
        shared_storage: &mut Nested,
        _context: &mut (),
    ) {
        shared_storage
            .trace
            .push(NestedRecord::InternalAfter(nesting));
    }
}

// MARK: Helpers

/// Return an initialized machine in `leaf` with an empty trace.
fn machine_at(leaf: Leaf) -> InitializedStateMachine<Tree> {
    let mut machine = Tree::default().uninitialized_state_machine().init(&mut ());

    if leaf != Leaf::A1 {
        machine.handle(Input::Go(leaf), &mut ());
    }

    machine.inner_mut().trace.clear();

    machine
}

/// Return a machine terminated from `leaf` with an empty trace.
fn terminated_at(leaf: Leaf) -> InitializedStateMachine<Tree> {
    let mut machine = machine_at(leaf);

    machine.handle(Input::Finish, &mut ());
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
    let uninitialized = Tree::default().uninitialized_state_machine();

    assert_eq!(uninitialized.trace, []);
    assert_eq!(uninitialized.state(), &TreeState::A1);

    let machine = uninitialized.init(&mut ());

    assert_eq!(
        machine.trace,
        [Record::Entry(Node::A), Record::Entry(Node::A1)],
    );
    assert_eq!(machine.state(), &TreeState::A1);
}

// MARK: Transitions

#[test]
fn sibling_transition_exits_and_enters_below_common_ancestor() {
    let mut machine = machine_at(Leaf::A1);

    assert_eq!(machine.handle(Input::Go(Leaf::B1), &mut ()), None);
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
    assert_eq!(machine.state(), &TreeState::B1 { visits: 2 });
}

#[test]
fn self_transition_exits_and_reenters_leaf() {
    let mut machine = machine_at(Leaf::A1);

    assert_eq!(machine.handle(Input::Go(Leaf::A1), &mut ()), None);
    assert_eq!(
        machine.trace,
        [
            Record::BeforeDispatch(Node::A1),
            Record::Handler(Node::A1),
            Record::AfterDispatch(Node::A1),
            Record::BeforeTransition(Leaf::A1, Leaf::A1),
            Record::Exit(Node::A1),
            Record::Entry(Node::A1),
            Record::AfterTransition(Leaf::A1, Leaf::A1),
        ],
    );
}

#[test]
fn top_level_transition_without_common_ancestor() {
    let mut machine = machine_at(Leaf::B1);

    assert_eq!(machine.handle(Input::Go(Leaf::C), &mut ()), None);
    assert_eq!(
        machine.trace,
        [
            Record::BeforeDispatch(Node::B1),
            Record::Handler(Node::B1),
            Record::AfterDispatch(Node::B1),
            Record::BeforeTransition(Leaf::B1, Leaf::C),
            Record::Exit(Node::B1),
            Record::Exit(Node::B),
            Record::Exit(Node::A),
            Record::Entry(Node::C),
            Record::AfterTransition(Leaf::B1, Leaf::C),
        ],
    );
    assert_eq!(machine.state(), &TreeState::C);
}

#[test]
fn external_transition_reenters_handling_superstate() {
    let mut machine = machine_at(Leaf::B1);

    assert_eq!(
        machine.handle(Input::ExternalFromB(Leaf::B2), &mut ()),
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
    assert_eq!(machine.state(), &TreeState::B2 { visits: 2 });
}

#[test]
fn external_transition_to_outside_behaves_locally() {
    let mut machine = machine_at(Leaf::B1);

    assert_eq!(machine.handle(Input::ExternalFromB(Leaf::C), &mut ()), None);
    assert_eq!(
        machine.trace,
        [
            dispatch_records(Node::B1).as_slice(),
            dispatch_records(Node::B).as_slice(),
            &[
                Record::BeforeTransition(Leaf::B1, Leaf::C),
                Record::Exit(Node::B1),
                Record::Exit(Node::B),
                Record::Exit(Node::A),
                Record::Entry(Node::C),
                Record::AfterTransition(Leaf::B1, Leaf::C),
            ],
        ]
        .concat(),
    );
    assert_eq!(machine.state(), &TreeState::C);
}

#[test]
fn exit_actions_see_source_and_entry_actions_see_target_storage() {
    let mut machine = machine_at(Leaf::B1);

    assert_eq!(machine.state(), &TreeState::B1 { visits: 2 });

    machine.handle(Input::Go(Leaf::B2), &mut ());

    // The exit of `B1` adds ten to the source, which is discarded, and the
    // entry of `B2` adds one to the fresh target.
    assert_eq!(machine.state(), &TreeState::B2 { visits: 1 });
}

#[test]
fn dropped_event_changes_nothing() {
    let mut machine = machine_at(Leaf::A1);

    assert_eq!(machine.handle(Input::Ignore, &mut ()), None);
    assert_eq!(
        machine.trace,
        [dispatch_records(Node::A1), dispatch_records(Node::A)].concat(),
    );
    assert_eq!(machine.state(), &TreeState::A1);
    assert!(!machine.is_terminated());
}

#[test]
fn internal_transition_hooks_nest_between_root_hooks() {
    let mut machine = Nested::default()
        .uninitialized_state_machine()
        .init(&mut ());

    machine.inner_mut().trace.clear();
    machine.handle((), &mut ());

    assert_eq!(
        machine.trace,
        [
            NestedRecord::RootBefore,
            NestedRecord::InternalBefore(1),
            NestedRecord::InternalBefore(2),
            NestedRecord::Exit,
            NestedRecord::Entry,
            NestedRecord::InternalAfter(2),
            NestedRecord::InternalAfter(1),
            NestedRecord::RootAfter,
        ],
    );
}

// MARK: Termination

#[test]
fn termination_runs_every_exit_action_and_reports_exit() {
    let mut machine = machine_at(Leaf::B1);

    assert_eq!(
        machine.handle(Input::Finish, &mut ()),
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
}

#[test]
fn terminated_machine_ignores_events() {
    let mut machine = machine_at(Leaf::C);

    machine.handle(Input::Finish, &mut ());

    let trace_at_termination = machine.trace.clone();

    assert_eq!(machine.handle(Input::Go(Leaf::A1), &mut ()), None);
    assert_eq!(machine.trace, trace_at_termination);
    assert_eq!(machine.state(), &TreeState::C);
    assert_eq!(machine.exit(), Some(&Finished::Done));
}

#[test]
fn restart_reenters_initial_state() {
    let mut machine = terminated_at(Leaf::C);

    machine.restart(&mut ());

    assert_eq!(
        machine.trace,
        [Record::Entry(Node::A), Record::Entry(Node::A1)],
    );
    assert_eq!(machine.exit(), None);
    assert!(!machine.is_terminated());
    assert_eq!(machine.state(), &TreeState::A1);
}

#[test]
fn restart_on_running_machine_does_nothing() {
    let mut machine = machine_at(Leaf::B1);

    machine.restart(&mut ());

    assert_eq!(machine.trace, []);
    assert_eq!(machine.state(), &TreeState::B1 { visits: 2 });
}

#[test]
fn into_exit_returns_owned_exit() {
    assert_eq!(terminated_at(Leaf::C).into_exit(), Some(Finished::Done));
    assert_eq!(machine_at(Leaf::C).into_exit(), None);
}
