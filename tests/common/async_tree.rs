//! An awaitable machine with the hierarchy `top > {A > {A1, B > {B1, B2}}, C}`.

use sotu::awaitable::{IntoStateMachine, State, Superstate};
use sotu::prelude::Outcome;

use super::pending_once::pending_once;
use super::tree::{Finished, Input, Leaf, Node, Record, leaf_node};

// MARK: Machine

#[derive(Default, Eq, PartialEq, Debug)]
pub struct AsyncTree {
    pub trace: Vec<Record>,
}

// Deliberately not `Clone`: the traits must never need to copy a state.
#[derive(Eq, PartialEq, Debug)]
pub enum AsyncTreeState {
    A1,
    B1 { visits: u32 },
    B2 { visits: u32 },
    C,
}

#[derive(Eq, PartialEq, Debug)]
pub enum AsyncTreeSuperstate<'sub> {
    A,
    B { visits: &'sub mut u32 },
}

// MARK: Helpers

pub fn async_state_for(leaf: Leaf) -> AsyncTreeState {
    match leaf {
        Leaf::A1 => AsyncTreeState::A1,
        Leaf::B1 => AsyncTreeState::B1 { visits: 0 },
        Leaf::B2 => AsyncTreeState::B2 { visits: 0 },
        Leaf::C => AsyncTreeState::C,
    }
}

pub fn async_leaf_of(state: &AsyncTreeState) -> Leaf {
    match state {
        AsyncTreeState::A1 => Leaf::A1,
        AsyncTreeState::B1 { .. } => Leaf::B1,
        AsyncTreeState::B2 { .. } => Leaf::B2,
        AsyncTreeState::C => Leaf::C,
    }
}

// MARK: Machine Implementation

impl IntoStateMachine for AsyncTree {
    type Event = Input;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = AsyncTreeState;
    type Superstate<'sub> = AsyncTreeSuperstate<'sub>;
    type Exit = Finished;

    fn initial() -> AsyncTreeState {
        AsyncTreeState::A1
    }
}

// MARK: Leaf States

impl State<AsyncTree> for AsyncTreeState {
    async fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut AsyncTree,
        event: &Input,
        _context: &mut (),
    ) -> Outcome<AsyncTreeState, Finished> {
        pending_once().await;

        shared_storage
            .trace
            .push(Record::Handler(leaf_node(async_leaf_of(self))));

        match *event {
            Input::Go(leaf) => Outcome::Transition(async_state_for(leaf)),
            Input::ExternalFromLeaf(leaf) => Outcome::ExternalTransition(async_state_for(leaf)),
            Input::Finish if matches!(self, AsyncTreeState::B1 { .. } | AsyncTreeState::C) => {
                Outcome::Exit(Finished::Done)
            }
            _ => Outcome::Super,
        }
    }

    fn superstate(&mut self) -> Option<AsyncTreeSuperstate<'_>> {
        match self {
            AsyncTreeState::A1 => Some(AsyncTreeSuperstate::A),
            AsyncTreeState::B1 { visits } | AsyncTreeState::B2 { visits } => {
                Some(AsyncTreeSuperstate::B { visits })
            }
            AsyncTreeState::C => None,
        }
    }
}

// MARK: Superstates

impl Superstate<AsyncTree> for AsyncTreeSuperstate<'_> {
    async fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut AsyncTree,
        event: &Input,
        _context: &mut (),
    ) -> Outcome<AsyncTreeState, Finished> {
        pending_once().await;

        match self {
            AsyncTreeSuperstate::A => {
                shared_storage.trace.push(Record::Handler(Node::A));

                match *event {
                    Input::GoFromA(leaf) => Outcome::Transition(async_state_for(leaf)),
                    Input::ExternalFromA(leaf) => {
                        Outcome::ExternalTransition(async_state_for(leaf))
                    }
                    _ => Outcome::Super,
                }
            }
            AsyncTreeSuperstate::B { .. } => {
                shared_storage.trace.push(Record::Handler(Node::B));

                match *event {
                    Input::ExternalFromB(leaf) => {
                        Outcome::ExternalTransition(async_state_for(leaf))
                    }
                    _ => Outcome::Super,
                }
            }
        }
    }

    fn superstate(self) -> Option<Self> {
        match self {
            AsyncTreeSuperstate::A => None,
            AsyncTreeSuperstate::B { .. } => Some(AsyncTreeSuperstate::A),
        }
    }

    fn same_state(&self, other: &AsyncTreeSuperstate<'_>) -> bool {
        matches!(
            (self, other),
            (AsyncTreeSuperstate::A, AsyncTreeSuperstate::A)
                | (AsyncTreeSuperstate::B { .. }, AsyncTreeSuperstate::B { .. })
        )
    }
}
