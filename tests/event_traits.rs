use sotu::event::SubEvent;
use test_strategy::proptest;

#[test]
fn unit_event_accepts_and_round_trips() {
    assert!(<() as SubEvent<()>>::accepts(&()));
    assert_eq!(<() as SubEvent<()>>::project(()), Ok(()));

    let () = <() as SubEvent<()>>::embed(());
}

#[proptest]
fn unit_context_leaves_context_untouched(value: u32) {
    let mut context = value;
    let () = sotu::event::unit_context(&mut context);

    proptest::prop_assert_eq!(context, value);
}
