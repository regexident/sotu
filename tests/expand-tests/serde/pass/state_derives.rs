use serde::{Deserialize, Serialize};
use sotu::prelude::*;

pub enum Event {
    Next,
}

pub struct Counter;

#[state_machine(initial = State::idle(), state(derive(Serialize, Deserialize)))]
impl Counter {
    #[state]
    fn idle(event: &Event) -> Outcome<State> {
        match event {
            Event::Next => Transition(State::counting(0)),
        }
    }

    #[state]
    fn counting(total: &mut u32, event: &Event) -> Outcome<State> {
        match event {
            Event::Next => {
                *total += 1;

                Handled
            }
        }
    }
}

fn assert_serde<T: Serialize + for<'de> Deserialize<'de>>() {}

fn main() {
    assert_serde::<State>();
}
