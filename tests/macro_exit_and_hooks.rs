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

use deferrer::{Deferrer, Tick};
use finisher::{Done, Finish, Finisher};

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
