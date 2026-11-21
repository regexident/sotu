mod common;

use common::trace;
use sotu::prelude::*;

// MARK: Machines

mod child {
    use std::cell::Cell;

    use sotu::prelude::*;

    use super::trace;

    /// The context of `Child`, borrowed from the parent's context.
    pub struct Inner<'ctx> {
        pub label: &'ctx str,
        pub mappings: &'ctx Cell<u32>,
    }

    /// A child whose leaf records the label of its context, whose superstate
    /// handles the events the leaf defers, and whose `before_dispatch` hook
    /// records the mapping count it observes.
    #[derive(Default)]
    pub struct Child {
        pub labels: Vec<String>,
        pub counts: Vec<u32>,
    }

    pub struct Read;

    pub struct Climb;

    #[derive(Event)]
    pub enum Event {
        Read(Read),
        Climb(Climb),
    }

    #[state_machine(
        initial = State::leaf(),
        context = Inner<'ctx>,
        before_dispatch = Self::before,
        after_dispatch = Self::after
    )]
    impl Child {
        fn before(
            &mut self,
            node: StateOrSuperstate<'_, State, Superstate>,
            _event: &Event,
            context: &mut Inner<'_>,
        ) {
            self.counts.push(context.mappings.get());

            trace::record(match node {
                StateOrSuperstate::State(_) => "child before leaf",
                StateOrSuperstate::Superstate(_) => "child before group",
            });
        }

        fn after(
            &mut self,
            node: StateOrSuperstate<'_, State, Superstate>,
            _event: &Event,
            _context: &mut Inner<'_>,
        ) {
            trace::record(match node {
                StateOrSuperstate::State(_) => "child after leaf",
                StateOrSuperstate::Superstate(_) => "child after group",
            });
        }

        #[state(superstate = group)]
        fn leaf(&mut self, event: &Event, context: &mut Inner<'_>) -> Outcome<State> {
            match event {
                Event::Read(_) => {
                    self.labels.push(context.label.to_owned());

                    Handled
                }
                Event::Climb(_) => Super,
            }
        }

        #[superstate]
        fn group() -> Outcome<State> {
            trace::record("child group handled");

            Handled
        }
    }
}

mod parent {
    use std::cell::Cell;

    use sotu::SubmachineNode;
    use sotu::prelude::*;

    use super::{child, trace};

    /// The context of `Parent`, from which each child callback borrows.
    pub struct Outer {
        pub label: &'static str,
        pub mappings: Cell<u32>,
    }

    /// Build the child's context from `outer`, counting the calls.
    pub fn map_context(outer: &mut Outer) -> child::Inner<'_> {
        outer.mappings.set(outer.mappings.get() + 1);

        child::Inner {
            label: outer.label,
            mappings: &outer.mappings,
        }
    }

    /// A parent that starts inside its mount and whose hooks record the
    /// mapping count they observe in the parent's context.
    #[derive(Default)]
    pub struct Parent {
        pub child: child::Child,
        pub observed: Vec<u32>,
    }

    #[derive(Event)]
    pub enum Event {
        Read(child::Read),
        Climb(child::Climb),
    }

    #[state_machine(
        initial = Superstate::reading(),
        context = Outer,
        before_dispatch = Self::before,
        after_dispatch = Self::after
    )]
    impl Parent {
        fn before(
            &mut self,
            node: StateOrSuperstate<'_, State, Superstate<'_>>,
            _event: &Event,
            context: &mut Outer,
        ) {
            self.observed.push(context.mappings.get());

            trace::record(match node {
                StateOrSuperstate::State(_) => "parent before child leaf",
                StateOrSuperstate::Superstate(Superstate::Reading(SubmachineNode::Inner(_))) => {
                    "parent before child group"
                }
                StateOrSuperstate::Superstate(Superstate::Reading(SubmachineNode::Mount)) => {
                    "parent before mount"
                }
            });
        }

        fn after(
            &mut self,
            node: StateOrSuperstate<'_, State, Superstate<'_>>,
            _event: &Event,
            context: &mut Outer,
        ) {
            self.observed.push(context.mappings.get());

            trace::record(match node {
                StateOrSuperstate::State(_) => "parent after child leaf",
                StateOrSuperstate::Superstate(Superstate::Reading(SubmachineNode::Inner(_))) => {
                    "parent after child group"
                }
                StateOrSuperstate::Superstate(Superstate::Reading(SubmachineNode::Mount)) => {
                    "parent after mount"
                }
            });
        }

        #[submachine(machine = child::Child, field = child, context = map_context)]
        fn reading(event: &Event) -> Outcome<State> {
            match event {
                Event::Read(_) | Event::Climb(_) => Handled,
            }
        }
    }
}

mod lamp {
    use sotu::prelude::*;

    /// A child without a context.
    #[derive(Default)]
    pub struct Lamp {
        pub toggles: u32,
    }

    pub struct Toggle;

    #[derive(Event)]
    pub enum Event {
        Toggle(Toggle),
    }

    #[state_machine(initial = State::off())]
    impl Lamp {
        #[state]
        fn off(&mut self, event: &Event) -> Outcome<State> {
            match event {
                Event::Toggle(_) => {
                    self.toggles += 1;

                    Handled
                }
            }
        }
    }
}

mod room {
    use sotu::prelude::*;

    use super::lamp;

    /// A parent with a `u32` context that mounts a child without one.
    #[derive(Default)]
    pub struct Room {
        pub lamp: lamp::Lamp,
    }

    pub struct Knock;

    #[derive(Event)]
    pub enum Event {
        Toggle(lamp::Toggle),
        Knock(Knock),
    }

    #[state_machine(initial = Superstate::lit(), context = u32)]
    impl Room {
        #[submachine(machine = lamp::Lamp, field = lamp, context = sotu::event::unit_context)]
        fn lit(event: &Event, context: &mut u32) -> Outcome<State> {
            match event {
                Event::Toggle(_) => Handled,
                Event::Knock(_) => {
                    *context += 1;

                    Handled
                }
            }
        }
    }
}

// MARK: Tests

/// Return an initialized `Parent` together with its context labeled `"x"`,
/// with the mapping count and the trace reset.
fn started_parent() -> (InitializedStateMachine<parent::Parent>, parent::Outer) {
    let mut context = parent::Outer {
        label: "x",
        mappings: std::cell::Cell::new(0),
    };

    let machine = parent::Parent::default()
        .uninitialized_state_machine()
        .init(&mut context);

    context.mappings.set(0);
    trace::take();

    (machine, context)
}

#[test]
fn child_receives_mapped_context() {
    let (mut machine, mut context) = started_parent();

    machine.handle(parent::Event::Read(child::Read), &mut context);

    assert_eq!(machine.child.labels, ["x"]);
}

#[test]
fn mapping_runs_once_per_child_callback() {
    let (mut machine, mut context) = started_parent();

    machine.handle(parent::Event::Climb(child::Climb), &mut context);

    assert_eq!(context.mappings.get(), 6);
    assert_eq!(machine.child.counts, [1, 4]);
}

#[test]
fn unit_context_mounts_unit_context_child() {
    let mut context = 0;
    let mut machine = room::Room::default()
        .uninitialized_state_machine()
        .init(&mut context);

    machine.handle(room::Event::Toggle(lamp::Toggle), &mut context);
    machine.handle(room::Event::Knock(room::Knock), &mut context);

    assert_eq!(machine.lamp.toggles, 1);
    assert_eq!(context, 1);
}

#[test]
fn parent_hooks_interleave_with_mapped_child_callbacks() {
    let (mut machine, mut context) = started_parent();

    machine.handle(parent::Event::Climb(child::Climb), &mut context);

    assert_eq!(
        trace::take(),
        [
            "parent before child leaf",
            "child before leaf",
            "child after leaf",
            "parent after child leaf",
            "parent before child group",
            "child before group",
            "child group handled",
            "child after group",
            "parent after child group",
        ]
    );
    assert_eq!(machine.observed, [0, 3, 3, 6]);
}
