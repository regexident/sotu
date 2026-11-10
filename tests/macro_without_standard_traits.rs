mod common;

mod logger {
    use sotu::prelude::*;

    pub struct Advance;

    pub struct Finish;

    #[derive(Event)]
    pub enum Event {
        Advance(Advance),
        Finish(Finish),
    }

    pub enum Done {
        Finished { count: u8 },
    }

    pub struct Logger {
        pub log: Vec<&'static str>,
    }

    #[state_machine(initial = Phase::idle(), state(name = Phase), exit = Done)]
    impl Logger {
        #[state(superstate = active, entry_action = enter_idle)]
        fn idle(&mut self, event: &Event) -> Outcome<Phase, Done> {
            match event {
                Event::Advance(_) => {
                    self.log.push("advance");

                    Transition(Phase::counting(1))
                }
                Event::Finish(_) => Super,
            }
        }

        #[state(superstate = active)]
        fn counting(&mut self, count: &mut u8, event: &Event) -> Outcome<Phase, Done> {
            match event {
                Event::Advance(_) => {
                    self.log.push("count");
                    *count += 1;

                    Handled
                }
                Event::Finish(_) => Exit(Done::Finished { count: *count }),
            }
        }

        #[superstate(exit_action = leave_active)]
        fn active(&mut self, event: &Event) -> Outcome<Phase, Done> {
            match event {
                Event::Advance(_) => Handled,
                Event::Finish(_) => {
                    self.log.push("finish early");

                    Exit(Done::Finished { count: 0 })
                }
            }
        }

        #[action]
        fn enter_idle(&mut self) {
            self.log.push("enter idle");
        }

        #[action]
        fn leave_active(&mut self) {
            self.log.push("leave active");
        }
    }
}

use core::fmt::Debug;

use logger::{Advance, Done, Event, Finish, Logger, Phase, Superstate};
use sotu::prelude::*;

#[test]
fn macro_machine_runs_without_standard_traits() {
    let mut machine = Logger { log: Vec::new() }
        .uninitialized_state_machine()
        .init(&mut ());

    assert!(matches!(machine.state(), Phase::Idle {}));

    machine.handle(Advance, &mut ());
    machine.handle(Advance, &mut ());

    assert!(matches!(machine.state(), Phase::Counting { count: 2 }));
    assert!(matches!(
        machine.handle(Finish, &mut ()),
        Some(Done::Finished { count: 2 })
    ));
    assert!(machine.is_terminated());

    machine.restart(&mut ());

    assert!(!machine.is_terminated());
    assert!(matches!(machine.state(), Phase::Idle {}));
    assert!(matches!(
        machine.handle(Finish, &mut ()),
        Some(Done::Finished { count: 0 })
    ));
    assert_eq!(
        machine.log,
        [
            "enter idle",
            "advance",
            "count",
            "leave active",
            "enter idle",
            "finish early",
            "leave active"
        ]
    );
}

#[test]
fn macro_types_implement_no_standard_traits() {
    common::assert_not_impl!(Logger: Clone);
    common::assert_not_impl!(Logger: Copy);
    common::assert_not_impl!(Logger: Debug);
    common::assert_not_impl!(Logger: PartialEq);
    common::assert_not_impl!(Logger: Default);

    common::assert_not_impl!(Event: Clone);
    common::assert_not_impl!(Event: Copy);
    common::assert_not_impl!(Event: Debug);
    common::assert_not_impl!(Event: PartialEq);
    common::assert_not_impl!(Event: Default);

    common::assert_not_impl!(Done: Clone);
    common::assert_not_impl!(Done: Copy);
    common::assert_not_impl!(Done: Debug);
    common::assert_not_impl!(Done: PartialEq);
    common::assert_not_impl!(Done: Default);

    common::assert_not_impl!(Phase: Clone);
    common::assert_not_impl!(Phase: Copy);
    common::assert_not_impl!(Phase: Debug);
    common::assert_not_impl!(Phase: PartialEq);
    common::assert_not_impl!(Phase: Default);

    common::assert_not_impl!(Superstate: Clone);
    common::assert_not_impl!(Superstate: Copy);
    common::assert_not_impl!(Superstate: Debug);
    common::assert_not_impl!(Superstate: PartialEq);
    common::assert_not_impl!(Superstate: Default);
}
