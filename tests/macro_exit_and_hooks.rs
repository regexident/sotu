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

mod recorder {
    use sotu::prelude::*;

    pub struct Recorder;

    pub struct Go;

    #[derive(Event)]
    pub enum Event {
        Go(Go),
    }

    fn label(
        node: &StateOrSuperstate<'_, State, ()>,
        a: &'static str,
        b: &'static str,
    ) -> &'static str {
        match node {
            StateOrSuperstate::State(State::A {}) => a,
            StateOrSuperstate::State(State::B {}) => b,
            StateOrSuperstate::Superstate(()) => "superstate",
        }
    }

    fn transition_label(source: &State, target: &State, expected: &'static str) -> &'static str {
        match (source, target) {
            (State::A {}, State::B {}) => expected,
            _ => "other transition",
        }
    }

    #[state_machine(
        initial = State::a(),
        context = Vec<&'static str>,
        before_dispatch = Self::before_dispatch,
        after_dispatch = Self::after_dispatch,
        before_transition = Self::before_transition,
        after_transition = Self::after_transition
    )]
    impl Recorder {
        fn before_dispatch(
            &mut self,
            node: StateOrSuperstate<'_, State, ()>,
            _event: &Event,
            context: &mut Vec<&'static str>,
        ) {
            context.push(label(&node, "before a", "before b"));
        }

        fn after_dispatch(
            &mut self,
            node: StateOrSuperstate<'_, State, ()>,
            _event: &Event,
            context: &mut Vec<&'static str>,
        ) {
            context.push(label(&node, "after a", "after b"));
        }

        fn before_transition(
            &mut self,
            source: &State,
            target: &State,
            context: &mut Vec<&'static str>,
        ) {
            context.push(transition_label(source, target, "before transition"));
        }

        fn after_transition(
            &mut self,
            source: &State,
            target: &State,
            context: &mut Vec<&'static str>,
        ) {
            context.push(transition_label(source, target, "after transition"));
        }

        #[state(exit_action = leave_a)]
        fn a(event: &Event, context: &mut Vec<&'static str>) -> Outcome<State> {
            match event {
                Event::Go(_) => {
                    context.push("a");

                    Transition(State::b())
                }
            }
        }

        #[state(entry_action = enter_b)]
        fn b(event: &Event) -> Outcome<State> {
            match event {
                Event::Go(_) => Handled,
            }
        }

        #[action]
        fn leave_a(context: &mut Vec<&'static str>) {
            context.push("exit a");
        }

        #[action]
        fn enter_b(context: &mut Vec<&'static str>) {
            context.push("enter b");
        }
    }
}

use deferrer::{Deferrer, Tick};
use finisher::{Done, Finish, Finisher};
use mover::{Go, Mover};
use recorder::Recorder;

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

#[test]
fn hooks_with_context_receive_context() {
    let mut trace = Vec::new();
    let mut machine = Recorder.uninitialized_state_machine().init(&mut trace);

    machine.handle(recorder::Go, &mut trace);

    assert_eq!(
        trace,
        [
            "before a",
            "a",
            "after a",
            "before transition",
            "exit a",
            "enter b",
            "after transition"
        ]
    );
}
