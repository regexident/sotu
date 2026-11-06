use sotu::Outcome;
use sotu::private::NodeOutcome;

// MARK: Node Outcome

#[test]
fn node_outcome_from_outcome_maps_every_variant() {
    assert_eq!(
        NodeOutcome::from(Outcome::<u8, u8>::Handled),
        NodeOutcome::Handled
    );
    assert_eq!(
        NodeOutcome::from(Outcome::<u8, u8>::Super),
        NodeOutcome::Super
    );
    assert_eq!(
        NodeOutcome::from(Outcome::<u8, u8>::Transition(1)),
        NodeOutcome::Transition(1)
    );
    assert_eq!(
        NodeOutcome::from(Outcome::<u8, u8>::ExternalTransition(2)),
        NodeOutcome::ExternalTransition {
            target: 2,
            handler_offset: 0,
        }
    );
    assert_eq!(
        NodeOutcome::from(Outcome::<u8, u8>::Exit(3)),
        NodeOutcome::Exit(3)
    );
}

#[test]
fn map_state_maps_only_targets() {
    assert_eq!(
        NodeOutcome::<u8, u8>::Transition(2).map_state(u16::from),
        NodeOutcome::Transition(2u16)
    );
    assert_eq!(
        NodeOutcome::<u8, u8>::ExternalTransition {
            target: 2,
            handler_offset: 3,
        }
        .map_state(u16::from),
        NodeOutcome::ExternalTransition {
            target: 2u16,
            handler_offset: 3,
        }
    );
    assert_eq!(
        NodeOutcome::<u8, u8>::DeferToMount { mount_offset: 4 }.map_state(u16::from),
        NodeOutcome::DeferToMount { mount_offset: 4 }
    );
    assert_eq!(
        NodeOutcome::<u8, u8>::Exit(5).map_state(u16::from),
        NodeOutcome::Exit(5)
    );
    assert_eq!(
        NodeOutcome::<u8, u8>::Handled.map_state(u16::from),
        NodeOutcome::Handled
    );
    assert_eq!(
        NodeOutcome::<u8, u8>::Super.map_state(u16::from),
        NodeOutcome::Super
    );
}
