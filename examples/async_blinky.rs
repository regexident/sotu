//! The `blinky` machine with asynchronous entry and exit actions.
//!
//! The hierarchy is the same as in `blinky.rs`: `led_on` and `led_off` are
//! leaves of the superstate `blinking`, and `not_blinking` is a top-level
//! state. Declaring a single `async fn` in the `impl` block makes the whole
//! machine awaitable, so `init` and `handle` return futures. Handlers stay
//! plain functions, and the actions are `async fn`s, which is where a real
//! machine would drive a pin or wait on a peripheral.
//!
//! The futures need an executor. This example uses `block_on` from the
//! `futures` crate, and the library itself does not depend on one.
//!
//! Run it with `cargo run --example async_blinky --features async`. The output
//! is the same as the output of `blinky`:
//!
//! ```plain
//! enter blinking
//! enter led_on
//! exit led_on
//! enter led_off
//! exit led_off
//! enter led_on
//! exit led_on
//! exit blinking
//! enter not_blinking
//! exit not_blinking
//! enter blinking
//! enter led_on
//! ```

use futures::executor::block_on;
use sotu::prelude::*;

struct TimerElapsed;

struct ButtonPressed;

#[derive(Event)]
enum Event {
    TimerElapsed(TimerElapsed),
    ButtonPressed(ButtonPressed),
}

struct Blinky;

#[state_machine(initial = Superstate::blinking())]
impl Blinky {
    #[state(superstate = blinking, entry_action = enter_led_on, exit_action = exit_led_on)]
    fn led_on(event: &Event) -> Outcome<State> {
        match event {
            Event::TimerElapsed(_) => Transition(State::led_off()),
            Event::ButtonPressed(_) => Super,
        }
    }

    #[state(superstate = blinking, entry_action = enter_led_off, exit_action = exit_led_off)]
    fn led_off(event: &Event) -> Outcome<State> {
        match event {
            Event::TimerElapsed(_) => Transition(State::led_on()),
            Event::ButtonPressed(_) => Super,
        }
    }

    #[superstate(
        initial = State::led_on(),
        entry_action = enter_blinking,
        exit_action = exit_blinking
    )]
    fn blinking(event: &Event) -> Outcome<State> {
        match event {
            Event::ButtonPressed(_) => Transition(State::not_blinking()),
            Event::TimerElapsed(_) => Super,
        }
    }

    #[state(entry_action = enter_not_blinking, exit_action = exit_not_blinking)]
    fn not_blinking(event: &Event) -> Outcome<State> {
        match event {
            Event::ButtonPressed(_) => Transition(State::led_on()),
            Event::TimerElapsed(_) => Handled,
        }
    }

    #[action]
    async fn enter_blinking() {
        println!("enter blinking");
    }

    #[action]
    async fn exit_blinking() {
        println!("exit blinking");
    }

    #[action]
    async fn enter_led_on() {
        println!("enter led_on");
    }

    #[action]
    async fn exit_led_on() {
        println!("exit led_on");
    }

    #[action]
    async fn enter_led_off() {
        println!("enter led_off");
    }

    #[action]
    async fn exit_led_off() {
        println!("exit led_off");
    }

    #[action]
    async fn enter_not_blinking() {
        println!("enter not_blinking");
    }

    #[action]
    async fn exit_not_blinking() {
        println!("exit not_blinking");
    }
}

fn main() {
    block_on(async {
        let mut machine = Blinky.uninitialized_state_machine().init(&mut ()).await;

        machine.handle(TimerElapsed, &mut ()).await;
        machine.handle(TimerElapsed, &mut ()).await;
        machine.handle(ButtonPressed, &mut ()).await;
        machine.handle(TimerElapsed, &mut ()).await;
        machine.handle(ButtonPressed, &mut ()).await;
    });
}
