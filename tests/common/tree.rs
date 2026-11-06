//! A hand-written machine with the hierarchy
//! `top > {A > {A1, B > {B1, B2}}, C}`.
//!
//! Every handler, action and hook appends to [`Tree::trace`], so tests can
//! compare the exact order of calls. The local `visits` counters of `B1`, `B2`
//! and `B` count entries (plus one) and exits (plus ten) so that tests can also
//! observe which values reach a superstate that borrows them.

use sotu::StateOrSuperstate;
use sotu::blocking::{IntoStateMachine, State, Superstate};
use sotu::prelude::Outcome;
use test_strategy::Arbitrary;

// MARK: Vocabulary

#[derive(Copy, Clone, Eq, PartialEq, Debug, Arbitrary)]
pub enum Leaf {
    A1,
    B1,
    B2,
    C,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Node {
    A,
    B,
    A1,
    B1,
    B2,
    C,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Record {
    BeforeDispatch(Node),
    Handler(Node),
    AfterDispatch(Node),
    BeforeTransition(Leaf, Leaf),
    AfterTransition(Leaf, Leaf),
    Entry(Node),
    Exit(Node),
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Arbitrary)]
pub enum Input {
    Go(Leaf),
    GoFromA(Leaf),
    ExternalFromA(Leaf),
    ExternalFromB(Leaf),
    ExternalFromLeaf(Leaf),
    Finish,
    Ignore,
}

#[derive(Eq, PartialEq, Debug)]
pub enum Finished {
    Done,
}

// MARK: Machine

#[derive(Default, Eq, PartialEq, Debug)]
pub struct Tree {
    pub trace: Vec<Record>,
}

// Deliberately not `Clone`: the traits must never need to copy a state.
#[derive(Eq, PartialEq, Debug)]
pub enum TreeState {
    A1,
    B1 { visits: u32 },
    B2 { visits: u32 },
    C,
}

#[derive(Eq, PartialEq, Debug)]
pub enum TreeSuperstate<'sub> {
    A,
    B { visits: &'sub mut u32 },
}

// MARK: Helpers

pub fn state_for(leaf: Leaf) -> TreeState {
    match leaf {
        Leaf::A1 => TreeState::A1,
        Leaf::B1 => TreeState::B1 { visits: 0 },
        Leaf::B2 => TreeState::B2 { visits: 0 },
        Leaf::C => TreeState::C,
    }
}

pub fn leaf_of(state: &TreeState) -> Leaf {
    match state {
        TreeState::A1 => Leaf::A1,
        TreeState::B1 { .. } => Leaf::B1,
        TreeState::B2 { .. } => Leaf::B2,
        TreeState::C => Leaf::C,
    }
}

pub fn leaf_node(leaf: Leaf) -> Node {
    match leaf {
        Leaf::A1 => Node::A1,
        Leaf::B1 => Node::B1,
        Leaf::B2 => Node::B2,
        Leaf::C => Node::C,
    }
}

pub fn parent_of(node: Node) -> Option<Node> {
    match node {
        Node::A1 | Node::B => Some(Node::A),
        Node::B1 | Node::B2 => Some(Node::B),
        Node::A | Node::C => None,
    }
}

pub fn depth_of(node: Node) -> usize {
    match node {
        Node::A | Node::C => 1,
        Node::A1 | Node::B => 2,
        Node::B1 | Node::B2 => 3,
    }
}

fn node_of(node: StateOrSuperstate<'_, TreeState, TreeSuperstate<'_>>) -> Node {
    match node {
        StateOrSuperstate::State(state) => leaf_node(leaf_of(state)),
        StateOrSuperstate::Superstate(TreeSuperstate::A) => Node::A,
        StateOrSuperstate::Superstate(TreeSuperstate::B { .. }) => Node::B,
    }
}

// MARK: Machine Implementation

impl IntoStateMachine for Tree {
    type Event = Input;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = TreeState;
    type Superstate<'sub> = TreeSuperstate<'sub>;
    type Exit = Finished;

    fn initial() -> TreeState {
        TreeState::A1
    }

    fn before_dispatch<'ctx, 'data: 'ctx>(
        &mut self,
        node: StateOrSuperstate<'_, TreeState, TreeSuperstate<'_>>,
        _event: &Input,
        _context: &mut (),
    ) {
        self.trace.push(Record::BeforeDispatch(node_of(node)));
    }

    fn after_dispatch<'ctx, 'data: 'ctx>(
        &mut self,
        node: StateOrSuperstate<'_, TreeState, TreeSuperstate<'_>>,
        _event: &Input,
        _context: &mut (),
    ) {
        self.trace.push(Record::AfterDispatch(node_of(node)));
    }

    fn before_transition<'ctx, 'data: 'ctx>(
        &mut self,
        source: &TreeState,
        target: &TreeState,
        _context: &mut (),
    ) {
        self.trace
            .push(Record::BeforeTransition(leaf_of(source), leaf_of(target)));
    }

    fn after_transition<'ctx, 'data: 'ctx>(
        &mut self,
        source: &TreeState,
        target: &TreeState,
        _context: &mut (),
    ) {
        self.trace
            .push(Record::AfterTransition(leaf_of(source), leaf_of(target)));
    }
}

// MARK: Leaf States

impl State<Tree> for TreeState {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Tree,
        event: &Input,
        _context: &mut (),
    ) -> Outcome<TreeState, Finished> {
        shared_storage
            .trace
            .push(Record::Handler(leaf_node(leaf_of(self))));

        match *event {
            Input::Go(leaf) => Outcome::Transition(state_for(leaf)),
            Input::ExternalFromLeaf(leaf) => Outcome::ExternalTransition(state_for(leaf)),
            Input::Finish if matches!(self, TreeState::B1 { .. } | TreeState::C) => {
                Outcome::Exit(Finished::Done)
            }
            _ => Outcome::Super,
        }
    }

    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Tree,
        _context: &mut (),
    ) {
        shared_storage
            .trace
            .push(Record::Entry(leaf_node(leaf_of(self))));

        if let TreeState::B1 { visits } | TreeState::B2 { visits } = self {
            *visits += 1;
        }
    }

    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Tree,
        _context: &mut (),
    ) {
        shared_storage
            .trace
            .push(Record::Exit(leaf_node(leaf_of(self))));

        if let TreeState::B1 { visits } | TreeState::B2 { visits } = self {
            *visits += 10;
        }
    }

    fn superstate(&mut self) -> Option<TreeSuperstate<'_>> {
        match self {
            TreeState::A1 => Some(TreeSuperstate::A),
            TreeState::B1 { visits } | TreeState::B2 { visits } => {
                Some(TreeSuperstate::B { visits })
            }
            TreeState::C => None,
        }
    }
}

// MARK: Superstates

impl Superstate<Tree> for TreeSuperstate<'_> {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Tree,
        event: &Input,
        _context: &mut (),
    ) -> Outcome<TreeState, Finished> {
        match self {
            TreeSuperstate::A => {
                shared_storage.trace.push(Record::Handler(Node::A));

                match *event {
                    Input::GoFromA(leaf) => Outcome::Transition(state_for(leaf)),
                    Input::ExternalFromA(leaf) => Outcome::ExternalTransition(state_for(leaf)),
                    _ => Outcome::Super,
                }
            }
            TreeSuperstate::B { .. } => {
                shared_storage.trace.push(Record::Handler(Node::B));

                match *event {
                    Input::ExternalFromB(leaf) => Outcome::ExternalTransition(state_for(leaf)),
                    _ => Outcome::Super,
                }
            }
        }
    }

    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Tree,
        _context: &mut (),
    ) {
        match self {
            TreeSuperstate::A => shared_storage.trace.push(Record::Entry(Node::A)),
            TreeSuperstate::B { visits } => {
                shared_storage.trace.push(Record::Entry(Node::B));
                **visits += 1;
            }
        }
    }

    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Tree,
        _context: &mut (),
    ) {
        match self {
            TreeSuperstate::A => shared_storage.trace.push(Record::Exit(Node::A)),
            TreeSuperstate::B { visits } => {
                shared_storage.trace.push(Record::Exit(Node::B));
                **visits += 10;
            }
        }
    }

    fn superstate(&mut self) -> Option<TreeSuperstate<'_>> {
        match self {
            TreeSuperstate::A => None,
            TreeSuperstate::B { .. } => Some(TreeSuperstate::A),
        }
    }

    fn same_state(&self, other: &TreeSuperstate<'_>) -> bool {
        matches!(
            (self, other),
            (TreeSuperstate::A, TreeSuperstate::A)
                | (TreeSuperstate::B { .. }, TreeSuperstate::B { .. })
        )
    }
}
