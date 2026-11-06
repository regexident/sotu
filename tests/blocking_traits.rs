mod common;

use core::convert::Infallible;

use common::tree::{Input, Leaf, Node, Record, Tree, TreeState, TreeSuperstate};
use sotu::Outcome;
use sotu::blocking::{IntoStateMachine, State, Superstate};
use sotu::private::{Dispatched, NodeOutcome};

// MARK: Probe Machine

#[derive(Eq, PartialEq, Debug)]
struct Probe;

#[derive(Eq, PartialEq, Debug)]
enum ProbeState {
    Only,
}

impl IntoStateMachine for Probe {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = ProbeState;
    type Superstate<'sub> = ();
    type Exit = Infallible;

    fn initial() -> ProbeState {
        ProbeState::Only
    }
}

impl State<Probe> for ProbeState {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Probe,
        _event: &(),
        _context: &mut (),
    ) -> Outcome<ProbeState, Infallible> {
        Outcome::Handled
    }
}

// MARK: Superstate Defaults

#[test]
fn unit_superstate_defers_and_matches_itself() {
    assert_eq!(
        <() as Superstate<Probe>>::call_handler(&mut (), &mut Probe, &(), &mut ()),
        Outcome::Super
    );
    assert!(<() as Superstate<Probe>>::same_state(&(), &()));
    assert!(<() as Superstate<Probe>>::superstate(&mut ()).is_none());
}

#[test]
fn superstate_default_dispatch_calls_handler() {
    let mut tree = Tree::default();

    let dispatched = TreeSuperstate::A.__dispatch(&mut tree, Input::GoFromA(Leaf::C), &mut ());

    assert_eq!(
        dispatched,
        Dispatched {
            outcome: NodeOutcome::Transition(TreeState::C),
            event: Input::GoFromA(Leaf::C),
        }
    );
    assert_eq!(tree.trace, vec![Record::Handler(Node::A)]);
}

// MARK: State Defaults

#[test]
fn same_state_compares_variants_only() {
    assert!(TreeState::B1 { visits: 1 }.same_state(&TreeState::B1 { visits: 9 }));
    assert!(!TreeState::B1 { visits: 0 }.same_state(&TreeState::B2 { visits: 0 }));
}

#[test]
fn default_dispatch_calls_handler_and_returns_event() {
    let mut tree = Tree::default();

    let dispatched = TreeState::A1.__dispatch(&mut tree, Input::Go(Leaf::C), &mut ());

    assert_eq!(
        dispatched,
        Dispatched {
            outcome: NodeOutcome::Transition(TreeState::C),
            event: Input::Go(Leaf::C),
        }
    );
    assert_eq!(tree.trace, vec![Record::Handler(Node::A1)]);
}

#[test]
fn default_dispatch_maps_external_transition_with_zero_offset() {
    let mut tree = Tree::default();

    let dispatched = TreeState::C.__dispatch(&mut tree, Input::ExternalFromLeaf(Leaf::B2), &mut ());

    assert_eq!(
        dispatched,
        Dispatched {
            outcome: NodeOutcome::ExternalTransition {
                target: TreeState::B2 { visits: 0 },
                handler_offset: 0,
            },
            event: Input::ExternalFromLeaf(Leaf::B2),
        }
    );
}

#[test]
fn default_is_addressed_accepts_every_event() {
    assert_eq!(
        TreeState::C.__is_addressed(Input::Finish),
        (true, Input::Finish)
    );
}

#[test]
fn default_internal_transition_hooks_do_nothing() {
    let mut tree = Tree::default();
    let mut source = TreeState::A1;
    let mut target = TreeState::C;

    assert_eq!(source.__internal_transition_nesting(&mut target, 1), 0);

    source.__before_internal_transition(&mut target, 1, 1, &mut tree, &mut ());
    source.__after_internal_transition(&mut target, 1, 1, &mut tree, &mut ());

    assert!(tree.trace.is_empty());
    assert_eq!(source, TreeState::A1);
    assert_eq!(target, TreeState::C);
}

// MARK: Node Outcome

#[test]
fn node_outcome_from_outcome_maps_every_variant() {
    assert_eq!(
        NodeOutcome::from(Outcome::<u8, u8>::Handled),
        NodeOutcome::Handled
    );
    assert_eq!(
        NodeOutcome::from(Outcome::<u8, u8>::Super),
        NodeOutcome::Super
    );
    assert_eq!(
        NodeOutcome::from(Outcome::<u8, u8>::Transition(1)),
        NodeOutcome::Transition(1)
    );
    assert_eq!(
        NodeOutcome::from(Outcome::<u8, u8>::ExternalTransition(2)),
        NodeOutcome::ExternalTransition {
            target: 2,
            handler_offset: 0,
        }
    );
    assert_eq!(
        NodeOutcome::from(Outcome::<u8, u8>::Exit(3)),
        NodeOutcome::Exit(3)
    );
}

#[test]
fn map_state_maps_only_targets() {
    assert_eq!(
        NodeOutcome::<u8, u8>::Transition(2).map_state(u16::from),
        NodeOutcome::Transition(2u16)
    );
    assert_eq!(
        NodeOutcome::<u8, u8>::ExternalTransition {
            target: 2,
            handler_offset: 3,
        }
        .map_state(u16::from),
        NodeOutcome::ExternalTransition {
            target: 2u16,
            handler_offset: 3,
        }
    );
    assert_eq!(
        NodeOutcome::<u8, u8>::DeferToMount { mount_offset: 4 }.map_state(u16::from),
        NodeOutcome::DeferToMount { mount_offset: 4 }
    );
    assert_eq!(
        NodeOutcome::<u8, u8>::Exit(5).map_state(u16::from),
        NodeOutcome::Exit(5)
    );
    assert_eq!(
        NodeOutcome::<u8, u8>::Handled.map_state(u16::from),
        NodeOutcome::Handled
    );
    assert_eq!(
        NodeOutcome::<u8, u8>::Super.map_state(u16::from),
        NodeOutcome::Super
    );
}
