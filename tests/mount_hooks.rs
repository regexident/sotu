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

    pub struct Flip;

    #[derive(Event)]
    pub enum Event {
        Ping(Ping),
        Flip(Flip),
    }

    /// Return the entries that `Leaf`'s transition hooks record before and
    /// after a transition from `source` to `target`.
    fn transition_entries(source: &State, target: &State) -> (&'static str, &'static str) {
        match (source, target) {
            (State::Idle {}, State::Active {}) => (
                "leaf before transition from idle to active",
                "leaf after transition from idle to active",
            ),
            _ => (
                "leaf before other transition",
                "leaf after other transition",
            ),
        }
    }

    #[state_machine(
        initial = State::idle(),
        before_dispatch = Self::before,
        after_dispatch = Self::after,
        before_transition = Self::before_transition,
        after_transition = Self::after_transition
    )]
    impl Leaf {
        fn before(&mut self, node: StateOrSuperstate<'_, State, ()>, _event: &Event) {
            trace::record(match node {
                StateOrSuperstate::State(State::Idle {}) => "leaf before leaf-leaf",
                StateOrSuperstate::State(State::Active {}) => "leaf before leaf-active",
                StateOrSuperstate::Superstate(()) => "leaf before superstate",
            });
        }

        fn after(&mut self, node: StateOrSuperstate<'_, State, ()>, _event: &Event) {
            trace::record(match node {
                StateOrSuperstate::State(State::Idle {}) => "leaf after leaf-leaf",
                StateOrSuperstate::State(State::Active {}) => "leaf after leaf-active",
                StateOrSuperstate::Superstate(()) => "leaf after superstate",
            });
        }

        fn before_transition(&mut self, source: &State, target: &State) {
            trace::record(transition_entries(source, target).0);
        }

        fn after_transition(&mut self, source: &State, target: &State) {
            trace::record(transition_entries(source, target).1);
        }

        #[state(entry_action = enter_idle, exit_action = exit_idle)]
        fn idle(event: &Event) -> Outcome<State> {
            match event {
                Event::Ping(_) => {
                    trace::record("leaf-leaf handled");

                    Handled
                }
                Event::Flip(_) => Transition(State::active()),
            }
        }

        #[state(entry_action = enter_active)]
        fn active() -> Outcome<State> {
            Handled
        }

        #[action]
        fn enter_idle() {
            trace::record("idle entered");
        }

        #[action]
        fn exit_idle() {
            trace::record("idle exited");
        }

        #[action]
        fn enter_active() {
            trace::record("active entered");
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

    pub struct Rest;

    #[derive(Event)]
    pub enum Event {
        Work(Work),
        Defer(Defer),
        Enter(Enter),
        Rest(Rest),
        Ping(leaf::Ping),
        Flip(leaf::Flip),
    }

    /// Return the entries that `Middle`'s hooks record before and after
    /// `node`, naming the node in `Middle`'s own types.
    fn entries(node: StateOrSuperstate<'_, State, Superstate<'_>>) -> (&'static str, &'static str) {
        match node {
            StateOrSuperstate::State(State::Busy {}) => {
                ("middle before middle-leaf", "middle after middle-leaf")
            }
            StateOrSuperstate::State(State::Rest {}) => {
                ("middle before middle-rest", "middle after middle-rest")
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

    /// Return the entries that `Middle`'s transition hooks record before and
    /// after a transition from `source` to `target`.
    fn transition_entries(source: &State, target: &State) -> (&'static str, &'static str) {
        match (source, target) {
            (State::Busy {}, State::Rest {}) => (
                "middle before transition from busy to rest",
                "middle after transition from busy to rest",
            ),
            (State::Nesting(leaf::State::Idle {}), State::Nesting(leaf::State::Active {})) => (
                "middle before transition from idle to active",
                "middle after transition from idle to active",
            ),
            _ => (
                "middle before other transition",
                "middle after other transition",
            ),
        }
    }

    #[state_machine(
        initial = State::busy(),
        before_dispatch = Self::before,
        after_dispatch = Self::after,
        before_transition = Self::before_transition,
        after_transition = Self::after_transition
    )]
    impl Middle {
        fn before(&mut self, node: StateOrSuperstate<'_, State, Superstate<'_>>, _event: &Event) {
            trace::record(entries(node).0);
        }

        fn after(&mut self, node: StateOrSuperstate<'_, State, Superstate<'_>>, _event: &Event) {
            trace::record(entries(node).1);
        }

        fn before_transition(&mut self, source: &State, target: &State) {
            trace::record(transition_entries(source, target).0);
        }

        fn after_transition(&mut self, source: &State, target: &State) {
            trace::record(transition_entries(source, target).1);
        }

        #[state(superstate = group, entry_action = enter_busy, exit_action = exit_busy)]
        fn busy(event: &Event) -> Outcome<State> {
            match event {
                Event::Work(_) => {
                    trace::record("middle-leaf handled");

                    Handled
                }
                Event::Enter(_) => Transition(Superstate::nesting()),
                Event::Rest(_) => Transition(State::rest()),
                Event::Defer(_) | Event::Ping(_) | Event::Flip(_) => {
                    trace::record("middle-leaf deferred");

                    Super
                }
            }
        }

        #[state(entry_action = enter_rest)]
        fn rest() -> Outcome<State> {
            Handled
        }

        #[superstate(entry_action = enter_group, exit_action = exit_group)]
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

        #[action]
        fn enter_busy() {
            trace::record("busy entered");
        }

        #[action]
        fn exit_busy() {
            trace::record("busy exited");
        }

        #[action]
        fn enter_rest() {
            trace::record("rest entered");
        }

        #[action]
        fn enter_group() {
            trace::record("group entered");
        }

        #[action]
        fn exit_group() {
            trace::record("group exited");
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

    pub struct Leave;

    pub struct Reenter;

    #[derive(Event)]
    pub enum Event {
        Work(middle::Work),
        Defer(middle::Defer),
        Enter(middle::Enter),
        Rest(middle::Rest),
        Ping(leaf::Ping),
        Flip(leaf::Flip),
        Tick(Tick),
        Leave(Leave),
        Reenter(Reenter),
    }

    /// Return the entries that `Root`'s hooks record before and after `node`.
    fn entries(node: StateOrSuperstate<'_, State, Superstate<'_>>) -> (&'static str, &'static str) {
        match node {
            StateOrSuperstate::State(State::Outside {}) => {
                ("root before outside", "root after outside")
            }
            StateOrSuperstate::State(State::Mount(middle::State::Busy {})) => {
                ("root before middle-leaf", "root after middle-leaf")
            }
            StateOrSuperstate::State(State::Mount(middle::State::Rest {})) => {
                ("root before middle-rest", "root after middle-rest")
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
        after_dispatch = Self::after,
        before_transition = Self::before_transition,
        after_transition = Self::after_transition
    )]
    impl Root {
        fn before(&mut self, node: StateOrSuperstate<'_, State, Superstate<'_>>, _event: &Event) {
            trace::record(entries(node).0);
        }

        fn after(&mut self, node: StateOrSuperstate<'_, State, Superstate<'_>>, _event: &Event) {
            trace::record(entries(node).1);
        }

        fn before_transition(&mut self, _source: &State, _target: &State) {
            trace::record("root before transition");
        }

        fn after_transition(&mut self, _source: &State, _target: &State) {
            trace::record("root after transition");
        }

        #[state(entry_action = enter_outside)]
        fn outside() -> Outcome<State> {
            Handled
        }

        #[submachine(machine = middle::Middle, field = middle)]
        fn mount(event: &Event) -> Outcome<State> {
            match event {
                Event::Tick(_) => {
                    trace::record("mount handled tick");

                    Handled
                }
                Event::Leave(_) => Transition(State::outside()),
                Event::Reenter(_) => ExternalTransition(State::mount(middle::State::busy())),
                _ => {
                    trace::record("mount saw other");

                    Handled
                }
            }
        }

        #[action]
        fn enter_outside() {
            trace::record("outside entered");
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

#[test]
fn child_transition_hooks_nest_inside_root_hooks() {
    let mut machine = busy_root();

    // Leaving `busy` exits both of its levels, as many as it lies deep inside
    // `Middle`, so the mount is retained by the narrowest margin.
    machine.handle(middle::Rest, &mut ());

    assert_eq!(
        trace::take(),
        [
            "root before middle-leaf",
            "middle before middle-leaf",
            "middle after middle-leaf",
            "root after middle-leaf",
            "root before transition",
            "middle before transition from busy to rest",
            "busy exited",
            "group exited",
            "rest entered",
            "middle after transition from busy to rest",
            "root after transition",
        ]
    );
    assert!(matches!(
        machine.state(),
        root::State::Mount(middle::State::Rest {})
    ));
}

#[test]
fn grandchild_transition_hooks_nest_three_levels() {
    let mut machine = nesting_root();

    machine.handle(leaf::Flip, &mut ());

    assert_eq!(
        trace::take(),
        [
            "root before leaf-leaf",
            "middle before leaf-leaf",
            "leaf before leaf-leaf",
            "leaf after leaf-leaf",
            "middle after leaf-leaf",
            "root after leaf-leaf",
            "root before transition",
            "middle before transition from idle to active",
            "leaf before transition from idle to active",
            "idle exited",
            "active entered",
            "leaf after transition from idle to active",
            "middle after transition from idle to active",
            "root after transition",
        ]
    );
    assert!(matches!(
        machine.state(),
        root::State::Mount(middle::State::Nesting(leaf::State::Active {}))
    ));
}

#[test]
fn leaving_mount_fires_no_child_transition_hooks() {
    let mut machine = busy_root();

    machine.handle(root::Leave, &mut ());

    assert_eq!(
        trace::take(),
        [
            "root before mount",
            "root after mount",
            "root before transition",
            "busy exited",
            "group exited",
            "outside entered",
            "root after transition",
        ]
    );
    assert!(matches!(machine.state(), root::State::Outside {}));
}

#[test]
fn external_reentry_of_mount_fires_no_child_transition_hooks() {
    let mut machine = busy_root();

    machine.handle(root::Reenter, &mut ());

    assert_eq!(
        trace::take(),
        [
            "root before mount",
            "root after mount",
            "root before transition",
            "busy exited",
            "group exited",
            "group entered",
            "busy entered",
            "root after transition",
        ]
    );
    assert!(matches!(
        machine.state(),
        root::State::Mount(middle::State::Busy {})
    ));
}
