#![cfg(all(feature = "serde", feature = "async"))]

mod common;

use common::interrupt::poll_then_drop;
use common::trace;
use futures::executor::block_on;
use lamp::{Brighten, Lamp};
use sotu::awaitable::{InitializedStateMachine, IntoStateMachineExt};

// Every action of the fixture suspends once before it records itself, so a
// future that is polled a fixed number of times is dropped at a known await
// point of a transition.

mod lamp {
    use serde::{Deserialize, Serialize};
    use sotu::prelude::*;

    use super::common::pending_once::pending_once;
    use super::common::trace;

    pub struct Brighten;

    #[derive(Event)]
    pub enum Event {
        Brighten(Brighten),
    }

    #[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
    pub enum Ended {
        Done,
    }

    #[derive(Default, Serialize, Deserialize)]
    pub struct Lamp {
        pub watts: u32,
    }

    #[state_machine(
        initial = State::dim(),
        exit = Ended,
        state(derive(Eq, PartialEq, Debug, Serialize, Deserialize)),
        awaitable
    )]
    impl Lamp {
        #[state(superstate = powered, entry_action = enter_dim, exit_action = exit_dim)]
        fn dim(event: &Event) -> Outcome<State, Ended> {
            match event {
                Event::Brighten(_) => Transition(State::bright()),
            }
        }

        #[state(superstate = powered, entry_action = enter_bright)]
        fn bright(event: &Event) -> Outcome<State, Ended> {
            match event {
                Event::Brighten(_) => Handled,
            }
        }

        #[superstate(entry_action = enter_powered)]
        fn powered(event: &Event) -> Outcome<State, Ended> {
            match event {
                Event::Brighten(_) => Handled,
            }
        }

        #[action]
        async fn enter_dim() {
            pending_once().await;
            trace::record("enter dim");
        }

        #[action]
        async fn exit_dim() {
            pending_once().await;
            trace::record("exit dim");
        }

        #[action]
        async fn enter_bright() {
            pending_once().await;
            trace::record("enter bright");
        }

        #[action]
        async fn enter_powered() {
            pending_once().await;
            trace::record("enter powered");
        }
    }
}

fn initialized_lamp() -> InitializedStateMachine<Lamp> {
    let machine = block_on(Lamp::default().uninitialized_state_machine().init(&mut ()));

    trace::take();

    machine
}

/// Return a lamp whose `Brighten` transition was dropped after `polls` polls.
fn interrupted_lamp(polls: usize) -> InitializedStateMachine<Lamp> {
    let mut machine = initialized_lamp();

    assert!(!poll_then_drop(machine.handle(Brighten, &mut ()), polls));

    machine
}

#[test]
fn pending_progress_is_not_serialized() -> Result<(), serde_json::Error> {
    let machine = interrupted_lamp(1);
    let value = serde_json::to_value(&machine)?;
    let mut keys: Vec<&str> = value
        .as_object()
        .expect("a machine serializes as an object")
        .keys()
        .map(String::as_str)
        .collect();

    keys.sort_unstable();

    assert_eq!(keys, ["exit", "shared_storage", "state"]);

    Ok(())
}
