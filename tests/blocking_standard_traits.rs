mod common;

use core::convert::Infallible;

use common::bare::{Bare, BareEvent, BareExit, BareState, BareSuperstate, Token};
use common::tree::{Input, Leaf, Tree};
use sotu::blocking::{
    InitializedStateMachine, IntoStateMachine, IntoStateMachineExt, State, StateMachine,
    UninitializedStateMachine,
};
use sotu::prelude::Outcome;

// MARK: Probe Machine

#[derive(Default, Eq, PartialEq, Debug)]
struct Probe {
    value: u8,
}

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

// MARK: Helpers

fn require_eq<T: Eq>() {}

/// Return an initialized `Tree` machine that handled `inputs`, with an empty
/// trace so that only its state and exit value can distinguish it.
fn tree_machine_after(inputs: &[Input]) -> StateMachine<Tree> {
    let mut machine = Tree::default().state_machine();

    machine.init(&mut ());

    for input in inputs {
        machine.handle(*input, &mut ());
    }

    machine.inner_mut().trace.clear();

    machine
}

// MARK: Debug

#[test]
fn state_machine_debug_lists_fields() {
    assert_eq!(
        format!("{:?}", Probe { value: 3 }.state_machine()),
        "StateMachine { shared_storage: Probe { value: 3 }, state: Only, exit: None, initialized: false }"
    );
}

#[test]
fn initialized_debug_lists_fields() {
    let uninitialized = Probe { value: 3 }.uninitialized_state_machine();

    assert_eq!(
        format!("{uninitialized:?}"),
        "UninitializedStateMachine { shared_storage: Probe { value: 3 }, state: Only, exit: None }"
    );

    let initialized = uninitialized.init(&mut ());

    assert_eq!(
        format!("{initialized:?}"),
        "InitializedStateMachine { shared_storage: Probe { value: 3 }, state: Only, exit: None }"
    );
}

// MARK: Equality

#[test]
fn equality_compares_storage_state_and_exit() {
    require_eq::<StateMachine<Probe>>();
    require_eq::<UninitializedStateMachine<Probe>>();
    require_eq::<InitializedStateMachine<Probe>>();

    assert_eq!(
        Probe { value: 3 }.state_machine(),
        Probe { value: 3 }.state_machine()
    );
    assert_ne!(
        Probe { value: 3 }.state_machine(),
        Probe { value: 4 }.state_machine()
    );

    let converted = StateMachine::from(
        Probe { value: 3 }
            .uninitialized_state_machine()
            .init(&mut ()),
    );

    assert_ne!(converted, Probe { value: 3 }.state_machine());

    assert_eq!(
        Probe { value: 3 }.uninitialized_state_machine(),
        Probe { value: 3 }.uninitialized_state_machine()
    );
    assert_ne!(
        Probe { value: 3 }.uninitialized_state_machine(),
        Probe { value: 4 }.uninitialized_state_machine()
    );

    assert_eq!(
        Probe { value: 3 }
            .uninitialized_state_machine()
            .init(&mut ()),
        Probe { value: 3 }
            .uninitialized_state_machine()
            .init(&mut ())
    );
    assert_ne!(
        Probe { value: 3 }
            .uninitialized_state_machine()
            .init(&mut ()),
        Probe { value: 4 }
            .uninitialized_state_machine()
            .init(&mut ())
    );

    let at_start = tree_machine_after(&[]);
    let at_leaf = tree_machine_after(&[Input::Go(Leaf::C)]);
    let finished = tree_machine_after(&[Input::Go(Leaf::C), Input::Finish]);

    assert_eq!(at_start, tree_machine_after(&[]));
    assert_ne!(at_start, at_leaf);
    assert_ne!(at_leaf, finished);
}

// MARK: Trait-Free Machines

#[test]
fn bare_machine_runs_without_standard_traits() {
    let mut machine = Bare { log: Vec::new() }.state_machine();

    machine.handle(BareEvent::Advance, &mut ());

    assert!(matches!(
        machine.handle(BareEvent::Stop, &mut ()),
        Some(BareExit::Stopped)
    ));

    machine.restart(&mut ());

    assert_eq!(machine.log, [9, 1, 11, 2, 12, 19, 9, 1]);
}

#[test]
fn bare_types_implement_no_standard_traits() {
    common::assert_not_impl!(Bare: Clone);
    common::assert_not_impl!(Bare: Copy);
    common::assert_not_impl!(Bare: core::fmt::Debug);
    common::assert_not_impl!(Bare: PartialEq);
    common::assert_not_impl!(Bare: Default);

    common::assert_not_impl!(BareEvent: Clone);
    common::assert_not_impl!(BareEvent: Copy);
    common::assert_not_impl!(BareEvent: core::fmt::Debug);
    common::assert_not_impl!(BareEvent: PartialEq);
    common::assert_not_impl!(BareEvent: Default);

    common::assert_not_impl!(BareExit: Clone);
    common::assert_not_impl!(BareExit: Copy);
    common::assert_not_impl!(BareExit: core::fmt::Debug);
    common::assert_not_impl!(BareExit: PartialEq);
    common::assert_not_impl!(BareExit: Default);

    common::assert_not_impl!(BareState: Clone);
    common::assert_not_impl!(BareState: Copy);
    common::assert_not_impl!(BareState: core::fmt::Debug);
    common::assert_not_impl!(BareState: PartialEq);
    common::assert_not_impl!(BareState: Default);

    common::assert_not_impl!(BareSuperstate<'static>: Clone);
    common::assert_not_impl!(BareSuperstate<'static>: Copy);
    common::assert_not_impl!(BareSuperstate<'static>: core::fmt::Debug);
    common::assert_not_impl!(BareSuperstate<'static>: PartialEq);
    common::assert_not_impl!(BareSuperstate<'static>: Default);

    common::assert_not_impl!(Token: Clone);
    common::assert_not_impl!(Token: Copy);
    common::assert_not_impl!(Token: core::fmt::Debug);
    common::assert_not_impl!(Token: PartialEq);
    common::assert_not_impl!(Token: Default);
}
