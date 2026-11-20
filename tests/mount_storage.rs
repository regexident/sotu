mod common;

use common::{assert_not_impl, trace};
use sotu::prelude::*;
use test_strategy::proptest;

mod door {
    use sotu::prelude::*;

    /// A child that can only be created through a factory.
    pub struct Door {
        pub id: u32,
        pub entries: u32,
    }

    #[derive(Event)]
    pub enum Event {}

    #[state_machine(initial = State::opened(), context = u32)]
    impl Door {
        #[state(superstate = frame, entry_action = count)]
        fn opened(event: &Event) -> Outcome<State> {
            match *event {}
        }

        #[superstate]
        fn frame(event: &Event) -> Outcome<State> {
            match *event {}
        }

        #[action]
        fn count(&mut self) {
            self.entries += 1;
        }
    }
}

mod logger {
    use sotu::prelude::*;

    use super::trace;

    /// A child kept in a persistent field, whose drop is recorded.
    pub struct Logger {
        pub entries: u32,
    }

    impl Drop for Logger {
        fn drop(&mut self) {
            trace::record("logger dropped");
        }
    }

    #[derive(Event)]
    pub enum Event {}

    #[state_machine(initial = State::writing(), context = u32)]
    impl Logger {
        #[state(entry_action = count)]
        fn writing(event: &Event) -> Outcome<State> {
            match *event {}
        }

        #[action]
        fn count(&mut self) {
            self.entries += 1;
        }
    }
}

mod house {
    use sotu::prelude::*;

    use super::{door, logger};

    pub struct House {
        pub next_id: u32,
        pub factory_calls: u32,
        pub door: Option<door::Door>,
        pub log: logger::Logger,
    }

    impl House {
        pub fn new(next_id: u32) -> Self {
            Self {
                next_id,
                factory_calls: 0,
                door: None,
                log: logger::Logger { entries: 0 },
            }
        }
    }

    pub enum Event {
        Enter,
        Leave,
        Log,
    }

    #[state_machine(initial = State::idle(), context = u32)]
    impl House {
        #[state]
        fn idle(event: &Event) -> Outcome<State> {
            match event {
                Event::Enter => Transition(Superstate::loading()),
                Event::Log => Transition(Superstate::logging()),
                Event::Leave => Handled,
            }
        }

        #[submachine(machine = door::Door, field = door, factory = Self::make_door)]
        fn loading(event: &Event) -> Outcome<State> {
            match event {
                Event::Leave => Transition(State::idle()),
                Event::Enter | Event::Log => Handled,
            }
        }

        #[submachine(machine = logger::Logger, field = log)]
        fn logging(event: &Event) -> Outcome<State> {
            match event {
                Event::Leave => Transition(State::idle()),
                Event::Enter | Event::Log => Handled,
            }
        }

        fn make_door(&mut self, context: &mut u32) -> door::Door {
            self.factory_calls += 1;

            door::Door {
                id: self.next_id + *context,
                entries: 0,
            }
        }
    }
}

mod porch {
    use sotu::prelude::*;

    use super::door;

    /// A parent that starts inside its mount.
    pub struct Porch {
        pub factory_calls: u32,
        pub door: Option<door::Door>,
    }

    #[state_machine(initial = Superstate::loading(), context = u32)]
    impl Porch {
        #[submachine(machine = door::Door, field = door, factory = Self::make_door)]
        fn loading() -> Outcome<State> {
            Handled
        }

        fn make_door(&mut self, _context: &mut u32) -> door::Door {
            self.factory_calls += 1;

            door::Door { id: 0, entries: 0 }
        }
    }
}

assert_not_impl!(door::Door: Default);

#[test]
fn factory_receives_parent_storage_and_context() {
    let mut context = 3;
    let mut machine = house::House::new(7)
        .uninitialized_state_machine()
        .init(&mut context);

    machine.handle(house::Event::Enter, &mut context);

    assert_eq!(machine.door.as_ref().map(|door| door.id), Some(10));
}

#[proptest]
fn factory_runs_once_per_activation(
    #[strategy(proptest::collection::vec(proptest::bool::ANY, 0..=20))] operations: Vec<bool>,
) {
    let mut context = 0;
    let mut machine = house::House::new(0)
        .uninitialized_state_machine()
        .init(&mut context);
    let mut active = false;
    let mut activations = 0;

    for enter in operations {
        if enter {
            machine.handle(house::Event::Enter, &mut context);

            if !active {
                activations += 1;
            }

            active = true;
        } else {
            machine.handle(house::Event::Leave, &mut context);
            active = false;
        }
    }

    proptest::prop_assert_eq!(machine.factory_calls, activations);
}

#[test]
fn factory_skips_filled_field() {
    let mut context = 0;
    let mut machine = porch::Porch {
        factory_calls: 0,
        door: None,
    }
    .uninitialized_state_machine();

    machine.inner_mut().door = Some(door::Door { id: 42, entries: 0 });

    let machine = machine.init(&mut context);

    assert_eq!(machine.door.as_ref().map(|door| door.id), Some(42));
    assert_eq!(machine.factory_calls, 0);
}

#[test]
fn child_without_default_mounts_through_factory() {
    let mut context = 0;
    let mut machine = house::House::new(0)
        .uninitialized_state_machine()
        .init(&mut context);

    machine.handle(house::Event::Enter, &mut context);

    assert!(matches!(
        machine.state(),
        house::State::Loading(door::State::Opened {})
    ));
    assert_eq!(machine.door.as_ref().map(|door| door.entries), Some(1));
}

#[test]
fn persistent_field_survives_reactivation() {
    let mut context = 0;
    let mut machine = house::House::new(0)
        .uninitialized_state_machine()
        .init(&mut context);

    machine.handle(house::Event::Log, &mut context);
    machine.handle(house::Event::Leave, &mut context);
    machine.handle(house::Event::Log, &mut context);
    machine.handle(house::Event::Leave, &mut context);

    assert_eq!(machine.log.entries, 2);
    assert!(matches!(machine.state(), house::State::Idle {}));
    assert!(trace::take().is_empty());
}
