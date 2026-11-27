use serde::{Deserialize, Serialize};
use sotu::prelude::*;

pub enum Event {
    Next,
}

pub struct Lamp;

#[state_machine(
    initial = State::dim(),
    state(derive(Serialize, Deserialize)),
    superstate(derive(Serialize, Deserialize))
)]
impl Lamp {
    #[state(superstate = powered)]
    fn dim(event: &Event) -> Outcome<State> {
        match event {
            Event::Next => Super,
        }
    }

    #[superstate]
    fn powered(event: &Event) -> Outcome<State> {
        match event {
            Event::Next => Handled,
        }
    }
}

fn assert_serde<T: Serialize + for<'de> Deserialize<'de>>() {}

fn main() {
    assert_serde::<State>();
    assert_serde::<Superstate>();
}
