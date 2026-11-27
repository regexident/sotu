//! A blinking LED with a button.
//!
//! `led_on` and `led_off` are leaves of the superstate `blinking`. The timer
//! toggles between the two leaves, and `blinking` handles the button for both
//! of them by moving to `not_blinking`. `not_blinking` is a top-level state
//! that ignores the timer and returns to `led_on` on the next button press,
//! which enters `blinking` again.
//!
//! Every entry and exit action prints its name, so the output shows the order
//! in which the machine enters and leaves each level of the hierarchy. Leaving
//! `led_on` for `not_blinking` runs `exit led_on` before `exit blinking`, and
//! entering `led_on` from `not_blinking` runs `enter blinking` before
//! `enter led_on`.
//!
//! Run it with `cargo run --example blinky`. The output is:
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
    fn enter_blinking() {
        println!("enter blinking");
    }

    #[action]
    fn exit_blinking() {
        println!("exit blinking");
    }

    #[action]
    fn enter_led_on() {
        println!("enter led_on");
    }

    #[action]
    fn exit_led_on() {
        println!("exit led_on");
    }

    #[action]
    fn enter_led_off() {
        println!("enter led_off");
    }

    #[action]
    fn exit_led_off() {
        println!("exit led_off");
    }

    #[action]
    fn enter_not_blinking() {
        println!("enter not_blinking");
    }

    #[action]
    fn exit_not_blinking() {
        println!("exit not_blinking");
    }
}

fn main() {
    let mut machine = Blinky.uninitialized_state_machine().init(&mut ());

    machine.handle(TimerElapsed, &mut ());
    machine.handle(TimerElapsed, &mut ());
    machine.handle(ButtonPressed, &mut ());
    machine.handle(TimerElapsed, &mut ());
    machine.handle(ButtonPressed, &mut ());
}
