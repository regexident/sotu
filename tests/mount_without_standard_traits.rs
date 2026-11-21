mod common;

// MARK: Child Machine

mod child {
    use sotu::prelude::*;

    /// A child without `Default`, which only the parent's factory creates.
    pub struct Child {
        pub spins: u8,
    }

    pub struct Spin;

    pub struct Drain;

    #[derive(Event)]
    pub enum Event {
        Spin(Spin),
        Drain(Drain),
    }

    pub enum ChildExit {
        Drained { spins: u8 },
    }

    #[state_machine(initial = State::filling(), exit = ChildExit, context = Vec<u8>)]
    impl Child {
        #[state(superstate = cycle, entry_action = enter_filling, exit_action = exit_filling)]
        fn filling(event: &Event) -> Outcome<State, ChildExit> {
            match event {
                Event::Spin(_) => Transition(State::spinning()),
                Event::Drain(_) => Super,
            }
        }

        #[state(superstate = cycle, exit_action = exit_spinning)]
        fn spinning(&mut self, event: &Event, context: &mut Vec<u8>) -> Outcome<State, ChildExit> {
            match event {
                Event::Spin(_) => {
                    self.spins += 1;
                    context.push(6);

                    Handled
                }
                Event::Drain(_) => Super,
            }
        }

        #[superstate(entry_action = enter_cycle, exit_action = exit_cycle)]
        fn cycle(&mut self, event: &Event) -> Outcome<State, ChildExit> {
            match event {
                Event::Spin(_) => Handled,
                Event::Drain(_) => Exit(ChildExit::Drained { spins: self.spins }),
            }
        }

        #[action]
        fn enter_cycle(context: &mut Vec<u8>) {
            context.push(3);
        }

        #[action]
        fn enter_filling(context: &mut Vec<u8>) {
            context.push(4);
        }

        #[action]
        fn exit_filling(context: &mut Vec<u8>) {
            context.push(5);
        }

        #[action]
        fn exit_spinning(context: &mut Vec<u8>) {
            context.push(9);
        }

        #[action]
        fn exit_cycle(context: &mut Vec<u8>) {
            context.push(10);
        }
    }
}

// MARK: Parent Machine

mod parent {
    use sotu::prelude::*;

    use super::child;

    /// A parent that mounts the child through a factory, so neither machine
    /// needs `Default`.
    pub struct Parent {
        pub child: Option<child::Child>,
    }

    pub struct Start;

    pub struct Stop;

    #[derive(Event)]
    pub enum Event {
        Start(Start),
        Stop(Stop),
        Spin(child::Spin),
        Drain(child::Drain),
    }

    pub enum ParentExit {
        Stopped { spins: u8 },
    }

    #[state_machine(initial = State::idle(), exit = ParentExit, context = Vec<u8>)]
    impl Parent {
        #[state(entry_action = enter_idle)]
        fn idle(event: &Event) -> Outcome<State, ParentExit> {
            match event {
                Event::Start(_) => Transition(Superstate::running()),
                _ => Handled,
            }
        }

        #[submachine(
            machine = child::Child,
            field = child,
            factory = Self::make_child,
            exit_action = exit_running,
            exit_handler = child_exited
        )]
        fn running(event: &Event, context: &mut Vec<u8>) -> Outcome<State, ParentExit> {
            match event {
                Event::Stop(_) => {
                    context.push(7);

                    Handled
                }
                _ => Super,
            }
        }

        #[exit_handler]
        fn child_exited(
            exit: child::ChildExit,
            context: &mut Vec<u8>,
        ) -> Outcome<State, ParentExit> {
            match exit {
                child::ChildExit::Drained { spins } => {
                    context.push(8);

                    Exit(ParentExit::Stopped { spins })
                }
            }
        }

        #[action]
        fn enter_idle(context: &mut Vec<u8>) {
            context.push(1);
        }

        #[action]
        fn exit_running(context: &mut Vec<u8>) {
            context.push(11);
        }

        fn make_child(&mut self, context: &mut Vec<u8>) -> child::Child {
            context.push(2);

            child::Child { spins: 0 }
        }
    }
}

use core::fmt::Debug;

use child::{Child, ChildExit, Drain, Spin};
use parent::{Parent, ParentExit, Start, Stop};
use sotu::prelude::*;

// MARK: Tests

#[test]
fn mounted_machine_runs_without_standard_traits() {
    let mut log = Vec::new();
    let mut machine = Parent { child: None }
        .uninitialized_state_machine()
        .init(&mut log);

    assert!(matches!(machine.state(), parent::State::Idle {}));

    machine.handle(Start, &mut log);

    assert!(matches!(
        machine.state(),
        parent::State::Running(child::State::Filling {})
    ));

    machine.handle(Spin, &mut log);
    machine.handle(Spin, &mut log);

    assert!(matches!(
        machine.state(),
        parent::State::Running(child::State::Spinning {})
    ));
    assert_eq!(machine.child.as_ref().map(|child| child.spins), Some(1));

    // `Stop` is no child event, so the mount handles it without the child.
    machine.handle(Stop, &mut log);

    assert!(matches!(
        machine.handle(Drain, &mut log),
        Some(ParentExit::Stopped { spins: 1 })
    ));
    assert!(machine.is_terminated());
    assert!(machine.child.is_none());

    machine.restart(&mut log);

    assert!(!machine.is_terminated());
    assert!(matches!(machine.state(), parent::State::Idle {}));

    machine.handle(Start, &mut log);

    assert_eq!(machine.child.as_ref().map(|child| child.spins), Some(0));
    // Initialization enters `idle` (1). `Start` creates the child through the
    // factory (2) and enters `cycle` (3) and `filling` (4). The first `Spin`
    // exits `filling` (5), and the second is handled inside `spinning` (6).
    // `Stop` reaches the mount handler only (7). `Drain` reaches the exit
    // handler (8), whose exit terminates the parent through `spinning` (9),
    // `cycle` (10) and the mount (11). Restarting enters `idle` again, and
    // `Start` creates and enters a fresh child.
    assert_eq!(log, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 1, 2, 3, 4]);
}

#[test]
fn mounted_types_implement_no_standard_traits() {
    common::assert_not_impl!(Parent: Clone);
    common::assert_not_impl!(Parent: Copy);
    common::assert_not_impl!(Parent: Debug);
    common::assert_not_impl!(Parent: PartialEq);
    common::assert_not_impl!(Parent: Default);

    common::assert_not_impl!(parent::Event: Clone);
    common::assert_not_impl!(parent::Event: Copy);
    common::assert_not_impl!(parent::Event: Debug);
    common::assert_not_impl!(parent::Event: PartialEq);
    common::assert_not_impl!(parent::Event: Default);

    common::assert_not_impl!(ParentExit: Clone);
    common::assert_not_impl!(ParentExit: Copy);
    common::assert_not_impl!(ParentExit: Debug);
    common::assert_not_impl!(ParentExit: PartialEq);
    common::assert_not_impl!(ParentExit: Default);

    common::assert_not_impl!(parent::State: Clone);
    common::assert_not_impl!(parent::State: Copy);
    common::assert_not_impl!(parent::State: Debug);
    common::assert_not_impl!(parent::State: PartialEq);
    common::assert_not_impl!(parent::State: Default);

    common::assert_not_impl!(parent::Superstate: Clone);
    common::assert_not_impl!(parent::Superstate: Copy);
    common::assert_not_impl!(parent::Superstate: Debug);
    common::assert_not_impl!(parent::Superstate: PartialEq);
    common::assert_not_impl!(parent::Superstate: Default);

    common::assert_not_impl!(Child: Clone);
    common::assert_not_impl!(Child: Copy);
    common::assert_not_impl!(Child: Debug);
    common::assert_not_impl!(Child: PartialEq);
    common::assert_not_impl!(Child: Default);

    common::assert_not_impl!(child::Event: Clone);
    common::assert_not_impl!(child::Event: Copy);
    common::assert_not_impl!(child::Event: Debug);
    common::assert_not_impl!(child::Event: PartialEq);
    common::assert_not_impl!(child::Event: Default);

    common::assert_not_impl!(ChildExit: Clone);
    common::assert_not_impl!(ChildExit: Copy);
    common::assert_not_impl!(ChildExit: Debug);
    common::assert_not_impl!(ChildExit: PartialEq);
    common::assert_not_impl!(ChildExit: Default);

    common::assert_not_impl!(child::State: Clone);
    common::assert_not_impl!(child::State: Copy);
    common::assert_not_impl!(child::State: Debug);
    common::assert_not_impl!(child::State: PartialEq);
    common::assert_not_impl!(child::State: Default);

    common::assert_not_impl!(child::Superstate: Clone);
    common::assert_not_impl!(child::Superstate: Copy);
    common::assert_not_impl!(child::Superstate: Debug);
    common::assert_not_impl!(child::Superstate: PartialEq);
    common::assert_not_impl!(child::Superstate: Default);
}
