# sotu: state of the union

Composable hierarchical state machines for Rust.

## State Machine

The storage is an ordinary struct. Each function marked `#[state]` becomes a variant of the generated `State` enum, and every event is a type of its own.

```rust
use sotu::prelude::*;

#[derive(Default)]
struct Lamp {
    toggles: u32,
}

struct Toggle;

#[derive(Event)]
enum Event {
    Toggle(Toggle),
}

#[state_machine(initial = State::off(), state(derive(PartialEq, Debug)))]
impl Lamp {
    #[state]
    fn off(&mut self, event: &Event) -> Outcome<State> {
        match event {
            Event::Toggle(_) => {
                self.toggles += 1;

                Transition(State::on())
            }
        }
    }

    #[state]
    fn on(&mut self, event: &Event) -> Outcome<State> {
        match event {
            Event::Toggle(_) => {
                self.toggles += 1;

                Transition(State::off())
            }
        }
    }
}

let mut machine = Lamp::default().state_machine();

machine.handle(Toggle, &mut ());
machine.handle(Toggle, &mut ());

assert_eq!(machine.state(), &State::Off {});
assert_eq!(machine.toggles, 2);
```

## Hierarchical State Machine

A washer mounts a door while it loads. The door is a complete machine with its own exit point. The washer lists the door's `Close` event in its own event enum, and its `exit_handler` turns the door's exit into a transition of the washer. The washer holds the door in an `Option` field, so leaving the mount drops it.

```rust
mod door {
    use sotu::prelude::*;

    pub struct Close;

    #[derive(Event)]
    pub enum Event {
        Close(Close),
    }

    pub enum DoorExit {
        Closed,
    }

    #[derive(Default)]
    pub struct Door;

    #[state_machine(initial = State::opened(), exit = DoorExit)]
    impl Door {
        #[state]
        fn opened(event: &Event) -> Outcome<State, DoorExit> {
            match event {
                Event::Close(_) => Exit(DoorExit::Closed),
            }
        }
    }
}

mod washer {
    use sotu::prelude::*;

    use super::door;

    pub struct Start;
    pub struct Tick;

    #[derive(Event)]
    pub enum Event {
        Start(Start),
        Close(door::Close),
        Tick(Tick),
    }

    #[derive(Default)]
    pub struct Washer {
        pub door: Option<door::Door>,
    }

    #[state_machine(initial = State::idle())]
    impl Washer {
        #[state]
        fn idle(event: &Event) -> Outcome<State> {
            match event {
                Event::Start(_) => Transition(Superstate::loading()),
                Event::Close(_) | Event::Tick(_) => Super,
            }
        }

        #[submachine(machine = door::Door, field = door, exit_handler = door_exited)]
        fn loading(event: &Event) -> Outcome<State> {
            match event {
                Event::Start(_) | Event::Close(_) | Event::Tick(_) => Super,
            }
        }

        #[exit_handler]
        fn door_exited(exit: door::DoorExit) -> Outcome<State> {
            match exit {
                door::DoorExit::Closed => Transition(State::washing()),
            }
        }

        #[state]
        fn washing(event: &Event) -> Outcome<State> {
            match event {
                Event::Tick(_) => Transition(State::idle()),
                Event::Start(_) | Event::Close(_) => Super,
            }
        }
    }
}

use sotu::prelude::*;
use washer::{Start, State, Tick, Washer};

fn main() {
    let mut machine = Washer::default().state_machine();
    
    machine.handle(Start, &mut ());
    
    assert!(matches!(machine.state(), State::Loading(door::State::Opened {})));
    
    machine.handle(door::Close, &mut ());
    
    assert!(machine.door.is_none());
    assert!(matches!(machine.state(), State::Washing {}));
    
    machine.handle(Tick, &mut ());
    
    assert!(matches!(machine.state(), State::Idle {}));
}
```

While the door is mounted, the washer's state embeds the door's state.

## Feature flags

| Feature  | Default | Effect |
|----------|---------|--------|
| `macros` | yes | Re-exports `#[state_machine]`, its item attributes and `#[derive(Event)]`. |
| `async`  | no  | Adds the `awaitable` module and lets handlers, actions and hooks be `async fn`. |
| `serde`  | no  | Implements `Serialize` for the machine wrappers, and `Deserialize` for the uninitialized and lazily initialized ones. |

## License

Licensed under `MIT OR Apache-2.0`. See `LICENSE-MIT` and `LICENSE-APACHE`.
