#![cfg(feature = "serde")]
#![allow(dead_code)]

mod common;

use common::trace;
use sotu::prelude::*;
use washer::{Start, State, Washer};

mod door {
    use serde::{Deserialize, Serialize};
    use sotu::prelude::*;

    use super::trace;

    /// A child that can only be created through a factory.
    #[derive(Serialize, Deserialize)]
    pub struct Door {
        pub openings: u32,
    }

    pub struct Open;

    pub struct Close;

    #[derive(Event)]
    pub enum Event {
        Open(Open),
        Close(Close),
    }

    #[state_machine(
        initial = State::opened(),
        state(derive(Eq, PartialEq, Debug, Serialize, Deserialize))
    )]
    impl Door {
        #[state(entry_action = enter_opened)]
        fn opened(event: &Event) -> Outcome<State> {
            match event {
                Event::Open(_) => Handled,
                Event::Close(_) => Transition(State::ajar()),
            }
        }

        #[state]
        fn ajar(event: &Event) -> Outcome<State> {
            match event {
                Event::Open(_) => Transition(State::opened()),
                Event::Close(_) => Handled,
            }
        }

        #[action]
        fn enter_opened() {
            trace::record("enter opened");
        }
    }
}

mod washer {
    use serde::{Deserialize, Serialize};
    use sotu::prelude::*;

    use super::{door, trace};

    pub use door::{Close, Open};

    pub struct Start;

    pub struct Wash;

    pub struct Finish;

    #[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
    pub enum Ending {
        Done,
    }

    #[derive(Event)]
    pub enum Event {
        Start(Start),
        Wash(Wash),
        Finish(Finish),
        Open(Open),
        Close(Close),
    }

    #[derive(Default, Serialize, Deserialize)]
    pub struct Washer {
        pub factory_calls: u32,
        pub door: Option<door::Door>,
    }

    #[state_machine(
        initial = State::idle(),
        exit = Ending,
        state(derive(Eq, PartialEq, Debug, Serialize, Deserialize))
    )]
    impl Washer {
        #[state]
        fn idle(event: &Event) -> Outcome<State, Ending> {
            match event {
                Event::Start(_) => Transition(Superstate::loading()),
                _ => Super,
            }
        }

        #[submachine(
            machine = door::Door,
            field = door,
            factory = Self::make_door,
            entry_action = enter_loading
        )]
        fn loading(event: &Event) -> Outcome<State, Ending> {
            match event {
                Event::Wash(_) => Transition(State::washing()),
                _ => Super,
            }
        }

        #[state(entry_action = enter_washing)]
        fn washing(event: &Event) -> Outcome<State, Ending> {
            match event {
                Event::Finish(_) => Exit(Ending::Done),
                _ => Super,
            }
        }

        fn make_door(&mut self, _context: &mut ()) -> door::Door {
            self.factory_calls += 1;

            door::Door { openings: 0 }
        }

        #[action]
        fn enter_loading() {
            trace::record("enter loading");
        }

        #[action]
        fn enter_washing() {
            trace::record("enter washing");
        }
    }
}

#[test]
fn serialized_machine_has_three_fields() -> Result<(), serde_json::Error> {
    let mut machine = Washer::default().state_machine();

    machine.handle(Start, &mut ());

    let value = serde_json::to_value(&machine)?;
    let mut keys: Vec<&str> = value
        .as_object()
        .expect("a machine serializes as an object")
        .keys()
        .map(String::as_str)
        .collect();

    keys.sort_unstable();

    assert_eq!(keys, ["exit", "shared_storage", "state"]);

    let idle = serde_json::to_string(&Washer::default().state_machine())?;

    assert_eq!(
        idle,
        r#"{"shared_storage":{"factory_calls":0,"door":null},"state":{"Idle":{}},"exit":null}"#
    );

    Ok(())
}

#[test]
fn round_trip_keeps_mounted_instance() -> Result<(), serde_json::Error> {
    let mut machine = Washer::default().state_machine();

    machine.handle(Start, &mut ());

    machine
        .inner_mut()
        .door
        .as_mut()
        .expect("entering the mount creates the instance")
        .openings = 4;

    let restored: StateMachine<Washer> = serde_json::from_str(&serde_json::to_string(&machine)?)?;

    assert_eq!(restored.door.as_ref().map(|door| door.openings), Some(4));
    assert_eq!(restored.factory_calls, 1);
    assert_eq!(restored.state(), machine.state());
    assert_eq!(restored.state(), &State::Loading(door::State::Opened {}));

    Ok(())
}

