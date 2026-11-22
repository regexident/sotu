#![cfg(feature = "async")]

mod common;

use futures::executor::block_on;
use sotu::prelude::*;

// MARK: Async Handlers And Actions

mod blinky {
    use sotu::prelude::*;

    use crate::common::pending_once::pending_once;

    pub struct Timer;

    #[derive(Event)]
    pub enum Event {
        Timer(Timer),
    }

    #[derive(Default)]
    pub struct Blinky {
        pub trace: Vec<&'static str>,
    }

    #[state_machine(initial = State::led_on())]
    impl Blinky {
        #[superstate(entry_action = enter_blinking)]
        fn blinking() -> Outcome<State> {
            Super
        }

        #[state(superstate = blinking, entry_action = enter_led_on, exit_action = exit_led_on)]
        async fn led_on(event: &Event) -> Outcome<State> {
            pending_once().await;

            match event {
                Event::Timer(_) => Transition(State::led_off()),
            }
        }

        #[state(superstate = blinking, entry_action = enter_led_off)]
        async fn led_off(event: &Event) -> Outcome<State> {
            pending_once().await;

            match event {
                Event::Timer(_) => Transition(State::led_on()),
            }
        }

        #[action]
        async fn enter_blinking(&mut self) {
            pending_once().await;

            self.trace.push("enter blinking");
        }

        #[action]
        async fn enter_led_on(&mut self) {
            pending_once().await;

            self.trace.push("enter led_on");
        }

        #[action]
        fn exit_led_on(&mut self) {
            self.trace.push("exit led_on");
        }

        #[action]
        async fn enter_led_off(&mut self) {
            pending_once().await;

            self.trace.push("enter led_off");
        }
    }
}

#[test]
fn async_handlers_and_actions_run_in_order() {
    let mut machine = blinky::Blinky::default().state_machine();

    block_on(machine.init(&mut ()));

    assert_eq!(machine.trace, ["enter blinking", "enter led_on"]);

    block_on(machine.handle(blinky::Timer, &mut ()));

    assert_eq!(
        machine.trace,
        [
            "enter blinking",
            "enter led_on",
            "exit led_on",
            "enter led_off"
        ]
    );
    assert!(matches!(machine.state(), blinky::State::LedOff {}));
}

// MARK: Hooks

mod deferrer {
    use sotu::prelude::*;

    use crate::common::pending_once::pending_once;

    pub struct Tick;

    #[derive(Event)]
    pub enum Event {
        Tick(Tick),
    }

    #[derive(Default)]
    pub struct Deferrer {
        pub trace: Vec<&'static str>,
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

        async fn after(&mut self, node: StateOrSuperstate<'_, State, Superstate>, _event: &Event) {
            let entry = label(&node, "after leaf", "after group");

            pending_once().await;

            self.trace.push(entry);
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

#[test]
fn mixed_plain_and_async_hooks_fire() {
    let mut machine = deferrer::Deferrer::default().state_machine();

    block_on(machine.handle(deferrer::Tick, &mut ()));

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

// MARK: Awaitable Flag

mod flagged {
    use sotu::prelude::*;

    pub struct Press;

    #[derive(Event)]
    pub enum Event {
        Press(Press),
    }

    #[derive(Default)]
    pub struct Flagged {
        pub trace: Vec<&'static str>,
    }

    #[state_machine(initial = State::idle(), awaitable)]
    impl Flagged {
        #[state(exit_action = leave_idle)]
        fn idle(&mut self, event: &Event) -> Outcome<State> {
            match event {
                Event::Press(_) => {
                    self.trace.push("idle");

                    Transition(State::busy())
                }
            }
        }

        #[state(entry_action = enter_busy)]
        fn busy(event: &Event) -> Outcome<State> {
            match event {
                Event::Press(_) => Handled,
            }
        }

        #[action]
        fn leave_idle(&mut self) {
            self.trace.push("leave idle");
        }

        #[action]
        fn enter_busy(&mut self) {
            self.trace.push("enter busy");
        }
    }
}

#[test]
fn awaitable_flag_without_async_items() {
    let mut machine: sotu::awaitable::StateMachine<flagged::Flagged> =
        flagged::Flagged::default().state_machine();

    block_on(machine.handle(flagged::Press, &mut ()));

    assert_eq!(machine.trace, ["idle", "leave idle", "enter busy"]);
    assert!(matches!(machine.state(), flagged::State::Busy {}));
}

// MARK: Borrowed Superstate Storage

mod counter {
    use sotu::prelude::*;

    use crate::common::pending_once::pending_once;

    pub struct Bump;

    #[derive(Event)]
    pub enum Event {
        Bump(Bump),
    }

    #[derive(Default)]
    pub struct Counter {
        pub seen: Vec<u32>,
    }

    #[state_machine(initial = State::on(0))]
    impl Counter {
        #[superstate]
        fn outer(&mut self, counter: &u32, event: &Event) -> Outcome<State> {
            match event {
                Event::Bump(_) => {
                    self.seen.push(*counter);

                    Handled
                }
            }
        }

        #[superstate(superstate = outer)]
        async fn counting(counter: &mut u32, event: &Event) -> Outcome<State> {
            pending_once().await;

            match event {
                Event::Bump(_) => {
                    *counter += 1;

                    Super
                }
            }
        }

        #[state(superstate = counting, local_storage(counter: u32))]
        fn on(event: &Event) -> Outcome<State> {
            match event {
                Event::Bump(_) => Super,
            }
        }
    }
}

#[test]
fn borrowed_superstate_storage_in_awaitable_machine() {
    let mut machine = counter::Counter::default().state_machine();

    block_on(machine.handle(counter::Bump, &mut ()));
    block_on(machine.handle(counter::Bump, &mut ()));

    assert!(matches!(
        machine.state(),
        counter::State::On { counter: 2, .. }
    ));
    assert_eq!(machine.seen, [1, 2]);
}

// MARK: Termination

mod finisher {
    use sotu::prelude::*;

    use crate::common::pending_once::pending_once;

    pub struct Next;
    pub struct Finish;

    #[derive(Event)]
    pub enum Event {
        Next(Next),
        Finish(Finish),
    }

    #[derive(Eq, PartialEq, Debug)]
    pub enum Done {
        Finished,
    }

    #[derive(Default)]
    pub struct Finisher {
        pub trace: Vec<&'static str>,
    }

    #[state_machine(initial = State::running(), exit = Done)]
    impl Finisher {
        #[state(entry_action = enter_running)]
        fn running(event: &Event) -> Outcome<State, Done> {
            match event {
                Event::Next(_) => Transition(State::finishing()),
                Event::Finish(_) => Handled,
            }
        }

        #[state]
        async fn finishing(event: &Event) -> Outcome<State, Done> {
            pending_once().await;

            match event {
                Event::Next(_) => Handled,
                Event::Finish(_) => Exit(Done::Finished),
            }
        }

        #[action]
        async fn enter_running(&mut self) {
            pending_once().await;

            self.trace.push("enter running");
        }
    }
}

#[test]
fn awaitable_machine_terminates_and_restarts() {
    let mut machine = finisher::Finisher::default().state_machine();

    block_on(machine.handle(finisher::Next, &mut ()));

    assert!(matches!(machine.state(), finisher::State::Finishing {}));
    assert_eq!(
        block_on(machine.handle(finisher::Finish, &mut ())),
        Some(&finisher::Done::Finished)
    );
    assert!(machine.is_terminated());

    block_on(machine.restart(&mut ()));

    assert!(!machine.is_terminated());
    assert_eq!(machine.exit(), None);
    assert!(matches!(machine.state(), finisher::State::Running {}));
    assert_eq!(machine.trace, ["enter running", "enter running"]);
}

// MARK: Resumption

mod nested {
    use std::cell::Cell;

    use sotu::prelude::*;

    use crate::common::pending_once::pending_once;

    // The test harness runs each test on its own thread, so concurrent tests
    // never see each other's starts.
    std::thread_local! {
        static STARTS: Cell<u32> = const { Cell::new(0) };
    }

    /// Return how many actions and hooks have started on the current thread
    /// since the last [`reset_starts`].
    pub fn starts() -> u32 {
        STARTS.get()
    }

    /// Reset the start count of the current thread to zero.
    pub fn reset_starts() {
        STARTS.set(0);
    }

    /// Count a start, then suspend once before the step takes effect.
    async fn start() {
        STARTS.set(STARTS.get() + 1);
        pending_once().await;
    }

    pub struct Go;
    pub struct Ignore;

    #[derive(Event)]
    pub enum Event {
        Go(Go),
        Ignore(Ignore),
    }

    #[derive(Default)]
    pub struct Nested {
        pub trace: Vec<&'static str>,
    }

    // The handlers are plain, so a handled event is always dispatched on the
    // first poll and only the transition can be interrupted.
    #[state_machine(
        initial = State::inner_leaf(),
        before_transition = Self::before_transition,
        after_transition = Self::after_transition
    )]
    impl Nested {
        async fn before_transition(&mut self, _source: &State, _target: &State) {
            start().await;

            self.trace.push("before transition");
        }

        async fn after_transition(&mut self, _source: &State, _target: &State) {
            start().await;

            self.trace.push("after transition");
        }

        #[superstate(exit_action = exit_outer)]
        fn outer() -> Outcome<State> {
            Super
        }

        #[superstate(superstate = outer, exit_action = exit_inner)]
        fn inner() -> Outcome<State> {
            Super
        }

        #[state(superstate = inner, exit_action = exit_inner_leaf)]
        fn inner_leaf(event: &Event) -> Outcome<State> {
            match event {
                Event::Go(_) => Transition(State::top_leaf()),
                Event::Ignore(_) => Handled,
            }
        }

        #[state(entry_action = enter_top_leaf)]
        fn top_leaf(event: &Event) -> Outcome<State> {
            match event {
                Event::Go(_) | Event::Ignore(_) => Handled,
            }
        }

        #[action]
        async fn exit_outer(&mut self) {
            start().await;

            self.trace.push("exit outer");
        }

        #[action]
        async fn exit_inner(&mut self) {
            start().await;

            self.trace.push("exit inner");
        }

        #[action]
        async fn exit_inner_leaf(&mut self) {
            start().await;

            self.trace.push("exit inner_leaf");
        }

        #[action]
        async fn enter_top_leaf(&mut self) {
            start().await;

            self.trace.push("enter top_leaf");
        }
    }
}

/// Return an initialized `Nested` machine with an empty trace and the start
/// count reset.
fn initialized_nested() -> sotu::awaitable::InitializedStateMachine<nested::Nested> {
    let mut machine = block_on(
        nested::Nested::default()
            .uninitialized_state_machine()
            .init(&mut ()),
    );

    machine.inner_mut().trace.clear();
    nested::reset_starts();

    machine
}

#[test_strategy::proptest]
fn generated_machine_resumes_after_dropped_future(#[strategy(0usize..16)] polls: usize) {
    let mut reference = initialized_nested();

    block_on(reference.handle(nested::Go, &mut ()));

    let records = std::mem::take(&mut reference.inner_mut().trace);
    let step_starts = nested::starts();

    proptest::prop_assert_eq!(
        &records,
        &[
            "before transition",
            "exit inner_leaf",
            "exit inner",
            "exit outer",
            "enter top_leaf",
            "after transition"
        ]
    );

    let mut machine = initialized_nested();
    let completed = common::interrupt::poll_then_drop(machine.handle(nested::Go, &mut ()), polls);

    if !completed {
        // An unpolled future never dispatched the event, so it is handled
        // again. Otherwise the next call completes the interrupted transition
        // first, and the ignored event leaves no record.
        if polls == 0 {
            block_on(machine.handle(nested::Go, &mut ()));
        } else {
            block_on(machine.handle(nested::Ignore, &mut ()));
        }
    }

    // A future dropped after at least one poll interrupted exactly one
    // started step, which resuming runs again.
    let repeated = u32::from(!completed && polls > 0);

    let at_top_leaf = matches!(machine.state(), nested::State::TopLeaf {});

    proptest::prop_assert_eq!(&machine.trace, &records);
    proptest::prop_assert_eq!(nested::starts(), step_starts + repeated);
    proptest::prop_assert!(at_top_leaf);
    proptest::prop_assert!(!machine.is_terminated());
}
