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
