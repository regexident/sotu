//! A calculator that adds two numbers.
//!
//! Each state owns the numbers it needs as fields. `first` collects the digits
//! of the left operand, `second` remembers that operand while it collects the
//! digits of the right one, and `done` holds the sum. All three states are
//! leaves of the superstate `keypad`, which handles `Clear` for them by moving
//! back to `first` with an empty display.
//!
//! The handlers receive the fields of their own state as `&mut` parameters, and
//! a transition builds the target state by passing the new field values to its
//! constructor.
//!
//! Run it with `cargo run --example calculator`. The output is:
//!
//! ```plain
//! result: 42
//! result: 15
//! ```

use sotu::prelude::*;

struct Digit(u8);

struct Plus;

struct Equals;

struct Clear;

#[derive(Event)]
enum Event {
    Digit(Digit),
    Plus(Plus),
    Equals(Equals),
    Clear(Clear),
}

struct Calculator;

/// Append `digit` to the decimal number `value`.
fn append(value: i64, digit: u8) -> i64 {
    value.saturating_mul(10).saturating_add(i64::from(digit))
}

#[state_machine(initial = State::first(0))]
impl Calculator {
    #[superstate]
    fn keypad(event: &Event) -> Outcome<State> {
        match event {
            Event::Clear(_) => Transition(State::first(0)),
            Event::Digit(_) | Event::Plus(_) | Event::Equals(_) => Super,
        }
    }

    #[state(superstate = keypad)]
    fn first(value: &mut i64, event: &Event) -> Outcome<State> {
        match event {
            Event::Digit(Digit(digit)) => {
                *value = append(*value, *digit);

                Handled
            }
            Event::Plus(_) => Transition(State::second(*value, 0)),
            Event::Equals(_) => Handled,
            Event::Clear(_) => Super,
        }
    }

    #[state(superstate = keypad)]
    fn second(first: &mut i64, value: &mut i64, event: &Event) -> Outcome<State> {
        match event {
            Event::Digit(Digit(digit)) => {
                *value = append(*value, *digit);

                Handled
            }
            Event::Plus(_) => Transition(State::second(first.saturating_add(*value), 0)),
            Event::Equals(_) => Transition(State::done(first.saturating_add(*value))),
            Event::Clear(_) => Super,
        }
    }

    #[state(superstate = keypad)]
    fn done(result: &mut i64, event: &Event) -> Outcome<State> {
        match event {
            Event::Digit(Digit(digit)) => Transition(State::first(append(0, *digit))),
            Event::Plus(_) => Transition(State::second(*result, 0)),
            Event::Equals(_) => Handled,
            Event::Clear(_) => Super,
        }
    }
}

/// Feed every character of `keys` to the calculator as a key press.
///
/// Digits, `+` and `=` are keys, and every other character is ignored.
fn press(machine: &mut StateMachine<Calculator>, keys: &str) {
    for key in keys.chars() {
        match key.to_digit(10) {
            Some(digit) => {
                machine.handle(Digit(digit as u8), &mut ());
            }
            None if key == '+' => {
                machine.handle(Plus, &mut ());
            }
            None if key == '=' => {
                machine.handle(Equals, &mut ());
            }
            None => {}
        }
    }
}

/// Print the result on the display, if the calculator has computed one.
fn print_result(machine: &StateMachine<Calculator>) {
    if let State::Done { result } = machine.state() {
        println!("result: {result}");
    }
}

fn main() {
    let mut machine = Calculator.state_machine();

    press(&mut machine, "12+30=");
    print_result(&machine);

    machine.handle(Clear, &mut ());

    press(&mut machine, "7+8=");
    print_result(&machine);
}
