mod common;

use common::trace;
use sotu::prelude::*;

// MARK: Machines

mod leaf {
    use sotu::prelude::*;

    use super::trace;

    #[derive(Default)]
    pub struct Leaf;

    pub struct Ping;

    #[derive(Event)]
    pub enum Event {
        Ping(Ping),
    }

    #[state_machine(
        initial = State::idle(),
        before_dispatch = Self::before,
        after_dispatch = Self::after
    )]
    impl Leaf {
        fn before(&mut self, node: StateOrSuperstate<'_, State, ()>, _event: &Event) {
            trace::record(match node {
                StateOrSuperstate::State(State::Idle {}) => "leaf before leaf-leaf",
                StateOrSuperstate::Superstate(()) => "leaf before superstate",
            });
        }

        fn after(&mut self, node: StateOrSuperstate<'_, State, ()>, _event: &Event) {
            trace::record(match node {
                StateOrSuperstate::State(State::Idle {}) => "leaf after leaf-leaf",
                StateOrSuperstate::Superstate(()) => "leaf after superstate",
            });
        }

        #[state]
        fn idle(event: &Event) -> Outcome<State> {
            match event {
                Event::Ping(_) => trace::record("leaf-leaf handled"),
            }

            Handled
        }
    }
}

mod middle {
    use sotu::prelude::*;

    use super::{leaf, trace};

    #[derive(Default)]
    pub struct Middle {
        pub leaf: Option<leaf::Leaf>,
    }

    pub struct Work;

    pub struct Defer;

    pub struct Enter;

    #[derive(Event)]
    pub enum Event {
        Work(Work),
        Defer(Defer),
        Enter(Enter),
        Ping(leaf::Ping),
    }

    /// Return the entries that `Middle`'s hooks record before and after
    /// `node`, naming the node in `Middle`'s own types.
    fn entries(node: StateOrSuperstate<'_, State, Superstate<'_>>) -> (&'static str, &'static str) {
        match node {
            StateOrSuperstate::State(State::Busy {}) => {
                ("middle before middle-leaf", "middle after middle-leaf")
            }
            StateOrSuperstate::State(State::Nesting(_)) => {
                ("middle before leaf-leaf", "middle after leaf-leaf")
            }
            StateOrSuperstate::Superstate(Superstate::Group {}) => (
                "middle before middle-superstate",
                "middle after middle-superstate",
            ),
            StateOrSuperstate::Superstate(Superstate::Nesting(_)) => {
                ("middle before nesting", "middle after nesting")
            }
        }
    }

    #[state_machine(
        initial = State::busy(),
        before_dispatch = Self::before,
        after_dispatch = Self::after
    )]
    impl Middle {
        fn before(&mut self, node: StateOrSuperstate<'_, State, Superstate<'_>>, _event: &Event) {
            trace::record(entries(node).0);
        }

        fn after(&mut self, node: StateOrSuperstate<'_, State, Superstate<'_>>, _event: &Event) {
            trace::record(entries(node).1);
        }

        #[state(superstate = group)]
        fn busy(event: &Event) -> Outcome<State> {
            match event {
                Event::Work(_) => {
                    trace::record("middle-leaf handled");

                    Handled
                }
                Event::Enter(_) => Transition(Superstate::nesting()),
                Event::Defer(_) | Event::Ping(_) => {
                    trace::record("middle-leaf deferred");

                    Super
                }
            }
        }

        #[superstate]
        fn group(event: &Event) -> Outcome<State> {
            match event {
                Event::Defer(_) => trace::record("middle-superstate handled"),
                _ => trace::record("middle-superstate saw other"),
            }

            Handled
        }

        #[submachine(machine = leaf::Leaf, field = leaf)]
        fn nesting(event: &Event) -> Outcome<State> {
            match event {
                Event::Ping(_) => trace::record("nesting saw ping"),
                _ => trace::record("nesting saw other"),
            }

            Handled
        }
    }
}

mod root {
    use sotu::prelude::*;

    use super::{leaf, middle, trace};

    #[derive(Default)]
    pub struct Root {
        pub middle: Option<middle::Middle>,
    }

    pub struct Tick;

    #[derive(Event)]
    pub enum Event {
        Work(middle::Work),
        Defer(middle::Defer),
        Enter(middle::Enter),
        Ping(leaf::Ping),
        Tick(Tick),
    }

    /// Return the entries that `Root`'s hooks record before and after `node`.
    fn entries(node: StateOrSuperstate<'_, State, Superstate<'_>>) -> (&'static str, &'static str) {
        match node {
            StateOrSuperstate::State(State::Mount(middle::State::Busy {})) => {
                ("root before middle-leaf", "root after middle-leaf")
            }
            StateOrSuperstate::State(State::Mount(middle::State::Nesting(_))) => {
                ("root before leaf-leaf", "root after leaf-leaf")
            }
            StateOrSuperstate::Superstate(Superstate::Mount(SubmachineNode::Inner(
                middle::Superstate::Group {},
            ))) => (
                "root before middle-superstate",
                "root after middle-superstate",
            ),
            StateOrSuperstate::Superstate(Superstate::Mount(SubmachineNode::Inner(
                middle::Superstate::Nesting(_),
            ))) => ("root before nesting", "root after nesting"),
            StateOrSuperstate::Superstate(Superstate::Mount(SubmachineNode::Mount)) => {
                ("root before mount", "root after mount")
            }
        }
    }

    #[state_machine(
        initial = Superstate::mount(),
        before_dispatch = Self::before,
        after_dispatch = Self::after
    )]
    impl Root {
        fn before(&mut self, node: StateOrSuperstate<'_, State, Superstate<'_>>, _event: &Event) {
            trace::record(entries(node).0);
        }

        fn after(&mut self, node: StateOrSuperstate<'_, State, Superstate<'_>>, _event: &Event) {
            trace::record(entries(node).1);
        }

        #[submachine(machine = middle::Middle, field = middle)]
        fn mount(event: &Event) -> Outcome<State> {
            match event {
                Event::Tick(_) => trace::record("mount handled tick"),
                _ => trace::record("mount saw other"),
            }

            Handled
        }
    }
}

/// Return an initialized `Root` with `Middle` in `busy`, with an empty trace.
fn busy_root() -> InitializedStateMachine<root::Root> {
    let machine = root::Root::default()
        .uninitialized_state_machine()
        .init(&mut ());

    trace::take();

    machine
}

/// Return an initialized `Root` with `Middle` in `nesting` and `Leaf` in
/// `idle`, with an empty trace.
fn nesting_root() -> InitializedStateMachine<root::Root> {
    let mut machine = busy_root();

    machine.handle(middle::Enter, &mut ());
    trace::take();

    machine
}

// MARK: Tests

#[test]
fn child_dispatch_hooks_nest_inside_root_hooks() {
    let mut machine = busy_root();

    machine.handle(middle::Work, &mut ());

    assert_eq!(
        trace::take(),
        [
            "root before middle-leaf",
            "middle before middle-leaf",
            "middle-leaf handled",
            "middle after middle-leaf",
            "root after middle-leaf",
        ]
    );
}

#[test]
fn grandchild_dispatch_hooks_nest_three_levels() {
    let mut machine = nesting_root();

    machine.handle(leaf::Ping, &mut ());

    assert_eq!(
        trace::take(),
        [
            "root before leaf-leaf",
            "middle before leaf-leaf",
            "leaf before leaf-leaf",
            "leaf-leaf handled",
            "leaf after leaf-leaf",
            "middle after leaf-leaf",
            "root after leaf-leaf",
        ]
    );
    assert!(matches!(
        machine.state(),
        root::State::Mount(middle::State::Nesting(leaf::State::Idle {}))
    ));
}

#[test]
fn child_dispatch_hooks_see_child_types() {
    let mut machine = busy_root();

    machine.handle(middle::Defer, &mut ());

    assert_eq!(
        trace::take(),
        [
            "root before middle-leaf",
            "middle before middle-leaf",
            "middle-leaf deferred",
            "middle after middle-leaf",
            "root after middle-leaf",
            "root before middle-superstate",
            "middle before middle-superstate",
            "middle-superstate handled",
            "middle after middle-superstate",
            "root after middle-superstate",
        ]
    );
}

#[test]
fn unaddressed_child_fires_no_child_hooks() {
    let mut machine = busy_root();

    machine.handle(root::Tick, &mut ());

    assert_eq!(
        trace::take(),
        [
            "root before mount",
            "mount handled tick",
            "root after mount"
        ]
    );
}
