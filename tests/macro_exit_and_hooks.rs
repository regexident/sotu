use sotu::prelude::*;

mod finisher {
    use sotu::prelude::*;

    pub struct Finisher;

    pub struct Finish;

    #[derive(Event)]
    pub enum Event {
        Finish(Finish),
    }

    pub enum Done {
        Finished,
    }

    #[state_machine(initial = State::running(), exit = Done)]
    impl Finisher {
        #[state]
        fn running(event: &Event) -> Outcome<State, Done> {
            match event {
                Event::Finish(_) => Exit(Done::Finished),
            }
        }
    }
}

mod deferrer {
    use sotu::prelude::*;

    pub struct Deferrer {
        pub trace: Vec<&'static str>,
    }

    pub struct Tick;

    #[derive(Event)]
    pub enum Event {
        Tick(Tick),
    }

    fn label(
        node: &StateOrSuperstate<'_, State, Superstate>,
        leaf: &'static str,
        group: &'static str,
    ) -> &'static str {
        match node {
            StateOrSuperstate::State(State::Leaf {}) => leaf,
            StateOrSuperstate::Superstate(Superstate::Group {}) => group,
        }
    }

    #[state_machine(
        initial = State::leaf(),
        before_dispatch = Self::before,
        after_dispatch = Self::after
    )]
    impl Deferrer {
        fn before(&mut self, node: StateOrSuperstate<'_, State, Superstate>, _event: &Event) {
            self.trace.push(label(&node, "before leaf", "before group"));
        }

        fn after(&mut self, node: StateOrSuperstate<'_, State, Superstate>, _event: &Event) {
            self.trace.push(label(&node, "after leaf", "after group"));
        }

        #[state(superstate = group)]
        fn leaf(&mut self, event: &Event) -> Outcome<State> {
            match event {
                Event::Tick(_) => {
                    self.trace.push("leaf");

                    Super
                }
            }
        }

        #[superstate]
        fn group(&mut self, event: &Event) -> Outcome<State> {
            match event {
                Event::Tick(_) => {
                    self.trace.push("group");

                    Handled
                }
            }
        }
    }
}

mod mover {
    use sotu::prelude::*;

    pub struct Mover {
        pub trace: Vec<&'static str>,
    }

    pub struct Go;

    #[derive(Event)]
    pub enum Event {
        Go(Go),
    }

    #[state_machine(
        initial = State::a(),
        before_transition = Self::before,
        after_transition = Self::after
    )]
    impl Mover {
        fn before(&mut self, source: &State, target: &State) {
            let label = match (source, target) {
                (State::A {}, State::B {}) => "before transition",
                _ => "before other transition",
            };

            self.trace.push(label);
        }

        fn after(&mut self, source: &State, target: &State) {
            let label = match (source, target) {
                (State::A {}, State::B {}) => "after transition",
                _ => "after other transition",
            };

            self.trace.push(label);
        }

        #[state(exit_action = leave_a)]
        fn a(event: &Event) -> Outcome<State> {
            match event {
                Event::Go(_) => Transition(State::b()),
            }
        }

        #[state(entry_action = enter_b)]
        fn b(event: &Event) -> Outcome<State> {
            match event {
                Event::Go(_) => Handled,
            }
        }

        #[action]
        fn leave_a(&mut self) {
            self.trace.push("exit a");
        }

        #[action]
        fn enter_b(&mut self) {
            self.trace.push("enter b");
        }
    }
}

use deferrer::{Deferrer, Tick};
use finisher::{Done, Finish, Finisher};
use mover::{Go, Mover};

#[test]
fn exit_terminates_macro_machine() {
    let mut machine = Finisher.state_machine();

    assert!(matches!(
        machine.handle(Finish, &mut ()),
        Some(&Done::Finished)
    ));
    assert!(machine.is_terminated());
}

#[test]
fn dispatch_hooks_surround_every_node() {
    let mut machine = Deferrer { trace: Vec::new() }.state_machine();

    machine.handle(Tick, &mut ());

    assert_eq!(
        machine.trace,
        [
            "before leaf",
            "leaf",
            "after leaf",
            "before group",
            "group",
            "after group"
        ]
    );
}

#[test]
fn transition_hooks_surround_transition() {
    let mut machine = Mover { trace: Vec::new() }.state_machine();

    machine.handle(Go, &mut ());

    assert_eq!(
        machine.trace,
        ["before transition", "exit a", "enter b", "after transition"]
    );
}
