use sotu::prelude::*;

mod deferring {
    use sotu::prelude::*;

    #[derive(Default)]
    pub struct Deferring {
        pub log: Vec<&'static str>,
    }

    pub struct Ping;

    #[derive(Event)]
    pub enum Event {
        Ping(Ping),
    }

    #[state_machine(initial = State::leaf())]
    impl Deferring {
        #[state(superstate = group)]
        fn leaf(&mut self, event: &Event) -> Outcome<State> {
            match event {
                Event::Ping(_) => {
                    self.log.push("leaf");

                    Super
                }
            }
        }

        #[superstate]
        fn group(&mut self, event: &Event) -> Outcome<State> {
            match event {
                Event::Ping(_) => {
                    self.log.push("superstate");

                    Handled
                }
            }
        }
    }
}

mod nested {
    use sotu::prelude::*;

    #[derive(Default)]
    pub struct Nested {
        pub log: Vec<&'static str>,
    }

    pub struct Leave;

    #[derive(Event)]
    pub enum Event {
        Leave(Leave),
    }

    #[state_machine(initial = State::leaf())]
    impl Nested {
        #[state(superstate = inner, entry_action = enter_leaf, exit_action = exit_leaf)]
        fn leaf(event: &Event) -> Outcome<State> {
            match event {
                Event::Leave(_) => Super,
            }
        }

        #[state(entry_action = enter_sibling)]
        fn sibling(event: &Event) -> Outcome<State> {
            match event {
                Event::Leave(_) => Handled,
            }
        }

        #[superstate(superstate = outer, entry_action = enter_inner, exit_action = exit_inner)]
        fn inner(event: &Event) -> Outcome<State> {
            match event {
                Event::Leave(_) => Super,
            }
        }

        #[superstate(entry_action = enter_outer, exit_action = exit_outer)]
        fn outer(event: &Event) -> Outcome<State> {
            match event {
                Event::Leave(_) => Transition(State::sibling()),
            }
        }

        #[action]
        fn enter_leaf(&mut self) {
            self.log.push("enter leaf");
        }

        #[action]
        fn exit_leaf(&mut self) {
            self.log.push("exit leaf");
        }

        #[action]
        fn enter_sibling(&mut self) {
            self.log.push("enter sibling");
        }

        #[action]
        fn enter_inner(&mut self) {
            self.log.push("enter inner");
        }

        #[action]
        fn exit_inner(&mut self) {
            self.log.push("exit inner");
        }

        #[action]
        fn enter_outer(&mut self) {
            self.log.push("enter outer");
        }

        #[action]
        fn exit_outer(&mut self) {
            self.log.push("exit outer");
        }
    }
}

mod blinker {
    use sotu::prelude::*;

    #[derive(Default)]
    pub struct Blinker;

    pub struct Tick;

    pub struct Boost;

    #[derive(Event)]
    pub enum Event {
        Tick(Tick),
        Boost(Boost),
    }

    #[state_machine(initial = Superstate::blinking())]
    impl Blinker {
        #[state(superstate = blinking, local_storage(counter: u32))]
        fn on(event: &Event) -> Outcome<State> {
            match event {
                Event::Tick(_) | Event::Boost(_) => Super,
            }
        }

        #[superstate(superstate = outer, initial = State::on(0))]
        fn blinking(counter: &mut u32, event: &Event) -> Outcome<State> {
            match event {
                Event::Tick(_) => {
                    *counter += 1;

                    Handled
                }
                Event::Boost(_) => Super,
            }
        }

        #[superstate]
        fn outer(counter: &mut u32, event: &Event) -> Outcome<State> {
            match event {
                Event::Boost(_) => {
                    *counter += 10;

                    Handled
                }
                Event::Tick(_) => Super,
            }
        }
    }
}

mod dishwasher {
    use sotu::prelude::*;

    #[derive(Default)]
    pub enum Resume {
        #[default]
        Idle,
        Washing,
    }

    #[derive(Default)]
    pub struct Dishwasher {
        pub resume: Resume,
    }

    pub struct Start;

    pub struct Opened;

    pub struct Closed;

    #[derive(Event)]
    pub enum Event {
        Start(Start),
        Opened(Opened),
        Closed(Closed),
    }

    #[state_machine(initial = State::idle())]
    impl Dishwasher {
        #[superstate]
        fn door_closed(event: &Event) -> Outcome<State> {
            match event {
                Event::Opened(_) => Transition(State::door_open()),
                Event::Start(_) | Event::Closed(_) => Super,
            }
        }

        #[state(superstate = door_closed, entry_action = remember_idle)]
        fn idle(event: &Event) -> Outcome<State> {
            match event {
                Event::Start(_) => Transition(State::washing()),
                Event::Opened(_) | Event::Closed(_) => Super,
            }
        }

        #[state(superstate = door_closed, entry_action = remember_washing)]
        fn washing(event: &Event) -> Outcome<State> {
            match event {
                Event::Start(_) | Event::Opened(_) | Event::Closed(_) => Super,
            }
        }

        #[state]
        fn door_open(&mut self, event: &Event) -> Outcome<State> {
            match (event, &self.resume) {
                (Event::Closed(_), Resume::Idle) => Transition(State::idle()),
                (Event::Closed(_), Resume::Washing) => Transition(State::washing()),
                (Event::Start(_) | Event::Opened(_), _) => Super,
            }
        }

        #[action]
        fn remember_idle(&mut self) {
            self.resume = Resume::Idle;
        }

        #[action]
        fn remember_washing(&mut self) {
            self.resume = Resume::Washing;
        }
    }
}

use blinker::{Blinker, Boost, Tick};
use deferring::{Deferring, Ping};
use dishwasher::{Closed, Dishwasher, Opened, Start};
use nested::{Leave, Nested};

#[test]
fn deferred_event_reaches_superstate() {
    let mut machine = Deferring::default().state_machine();

    machine.handle(Ping, &mut ());

    assert_eq!(machine.log, ["leaf", "superstate"]);
}

#[test]
fn nested_superstates_run_actions_in_order() {
    let mut machine = Nested::default()
        .uninitialized_state_machine()
        .init(&mut ());

    assert_eq!(machine.log, ["enter outer", "enter inner", "enter leaf"]);

    machine.inner_mut().log.clear();
    machine.handle(Leave, &mut ());

    assert_eq!(
        machine.log,
        ["exit leaf", "exit inner", "exit outer", "enter sibling"]
    );
}

#[test]
fn superstate_shares_leaf_local_storage() {
    let mut machine = Blinker.state_machine();

    machine.handle(Tick, &mut ());
    machine.handle(Tick, &mut ());

    assert!(matches!(
        machine.state(),
        blinker::State::On { counter: 2, .. }
    ));

    machine.handle(Boost, &mut ());

    assert!(matches!(
        machine.state(),
        blinker::State::On { counter: 12, .. }
    ));
}

#[test]
fn initial_constructor_builds_state() {
    assert!(matches!(
        blinker::Superstate::blinking(),
        blinker::State::On { counter: 0, .. }
    ));
}

#[test]
fn dishwasher_resumes_previous_state() {
    let mut washing = Dishwasher::default().state_machine();

    washing.handle(Start, &mut ());
    washing.handle(Opened, &mut ());

    assert!(matches!(washing.state(), dishwasher::State::DoorOpen {}));

    washing.handle(Closed, &mut ());

    assert!(matches!(washing.state(), dishwasher::State::Washing {}));

    let mut idle = Dishwasher::default().state_machine();

    idle.handle(Opened, &mut ());

    assert!(matches!(idle.state(), dishwasher::State::DoorOpen {}));

    idle.handle(Closed, &mut ());

    assert!(matches!(idle.state(), dishwasher::State::Idle {}));
}

#[test]
fn superstate_same_state_ignores_lifetimes() {
    let mut first_counter = 1;
    let first = blinker::Superstate::Blinking {
        counter: &mut first_counter,
    };

    let mut second_counter = 2;

    {
        let mut third_counter = 3;

        let second = blinker::Superstate::Blinking {
            counter: &mut second_counter,
        };

        let outer = blinker::Superstate::Outer {
            counter: &mut third_counter,
        };

        assert!(first.same_state(&second));
        assert!(!first.same_state(&outer));
    }
}
