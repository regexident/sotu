//! A machine whose types implement none of `Clone`, `Copy`, `Debug`,
//! `PartialEq` or `Default`, so that it compiles only while the runtime demands
//! none of those traits.
//!
//! Both leaves sit under `Group`, which borrows the local storage of the active
//! leaf. Entry actions push `1` (`First`), `2` (`Second`) or `9` (`Group`) to
//! [`Bare::log`], and exit actions push ten more than the entry value.

use sotu::blocking::{IntoStateMachine, State, Superstate};
use sotu::prelude::Outcome;

// MARK: Machine

pub struct Bare {
    pub log: Vec<u8>,
}

pub enum BareEvent {
    Advance,
    Stop,
}

pub enum BareExit {
    Stopped,
}

pub enum BareState {
    First { local: Token },
    Second { local: Token },
}

pub enum BareSuperstate<'sub> {
    Group { local: &'sub mut Token },
}

pub struct Token(pub u8);

// MARK: Machine Implementation

impl IntoStateMachine for Bare {
    type Event = BareEvent;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = BareState;
    type Superstate<'sub> = BareSuperstate<'sub>;
    type Exit = BareExit;

    fn initial() -> BareState {
        BareState::First { local: Token(0) }
    }
}

// MARK: Leaf States

impl State<Bare> for BareState {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Bare,
        event: &BareEvent,
        _context: &mut (),
    ) -> Outcome<BareState, BareExit> {
        match (event, self) {
            (BareEvent::Advance, BareState::First { .. }) => {
                Outcome::Transition(BareState::Second { local: Token(0) })
            }
            (BareEvent::Advance, BareState::Second { .. }) => {
                Outcome::Transition(BareState::First { local: Token(0) })
            }
            (BareEvent::Stop, _) => Outcome::Exit(BareExit::Stopped),
        }
    }

    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Bare,
        _context: &mut (),
    ) {
        match self {
            BareState::First { .. } => shared_storage.log.push(1),
            BareState::Second { .. } => shared_storage.log.push(2),
        }
    }

    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Bare,
        _context: &mut (),
    ) {
        match self {
            BareState::First { .. } => shared_storage.log.push(11),
            BareState::Second { .. } => shared_storage.log.push(12),
        }
    }

    fn superstate(&mut self) -> Option<BareSuperstate<'_>> {
        match self {
            BareState::First { local } | BareState::Second { local } => {
                Some(BareSuperstate::Group { local })
            }
        }
    }
}

// MARK: Superstates

impl Superstate<Bare> for BareSuperstate<'_> {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Bare,
        _event: &BareEvent,
        _context: &mut (),
    ) -> Outcome<BareState, BareExit> {
        Outcome::Super
    }

    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Bare,
        _context: &mut (),
    ) {
        shared_storage.log.push(9);
    }

    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Bare,
        _context: &mut (),
    ) {
        shared_storage.log.push(19);
    }

    fn same_state(&self, other: &BareSuperstate<'_>) -> bool {
        matches!(
            (self, other),
            (BareSuperstate::Group { .. }, BareSuperstate::Group { .. })
        )
    }
}
