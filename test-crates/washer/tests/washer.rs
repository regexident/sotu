use sotu::prelude::*;
use sotu_test_washer::{Start, State, Washer};

// MARK: Helpers

/// Return an initialized `Washer` in `loading` with the door in `opened`,
/// with an empty washer trace.
fn loading_washer() -> InitializedStateMachine<Washer> {
    let mut washer = Washer::default()
        .uninitialized_state_machine()
        .init(&mut ());

    washer.handle(Start, &mut ());
    washer.inner_mut().trace.clear();

    washer
}

// MARK: Tests

#[test]
fn start_enters_door_from_other_crate() {
    let mut washer = Washer::default()
        .uninitialized_state_machine()
        .init(&mut ());

    washer.handle(Start, &mut ());

    assert_eq!(
        washer.trace,
        [
            "before idle",
            "after idle",
            "transition idle -> loading/opened",
            "transitioned idle -> loading/opened",
        ]
    );
    assert!(matches!(
        washer.state(),
        State::Loading(sotu_test_door::State::Opened {})
    ));
    assert_eq!(washer.door().map(|door| door.openings), Some(1));
}

#[test]
fn shared_tick_reaches_door_then_mount() {
    let mut washer = loading_washer();

    washer.handle(sotu_test_door::Tick, &mut ());

    assert_eq!(
        washer.trace,
        [
            "before loading/opened",
            "after loading/opened",
            "before loading",
            "after loading",
        ]
    );
    assert!(matches!(
        washer.state(),
        State::Loading(sotu_test_door::State::Opened {})
    ));
    assert_eq!(washer.door().map(|door| door.openings), Some(1));
}

#[test]
fn close_exits_door_through_exit_handler() {
    let mut washer = loading_washer();

    washer.handle(sotu_test_door::Close, &mut ());

    assert_eq!(
        washer.trace,
        [
            "before loading/opened",
            "after loading/opened",
            "transition loading/opened -> washing",
            "transitioned loading/opened -> washing",
        ]
    );
    assert!(matches!(washer.state(), State::Washing {}));
    assert!(washer.door().is_none());
}

#[test]
fn door_hooks_see_door_types() {
    let mut washer = loading_washer();

    washer.handle(sotu_test_door::Open, &mut ());

    assert_eq!(
        washer.door().map(|door| door.trace.as_slice()),
        Some(["before opened", "after opened"].as_slice())
    );
}
