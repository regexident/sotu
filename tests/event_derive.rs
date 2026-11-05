use proptest::prop_assert_eq;
use sotu::Event;
use sotu::event::{SubEvent, Variant};
use test_strategy::{Arbitrary, proptest};

#[derive(Event, Debug, Clone, Copy, PartialEq, Arbitrary)]
enum Child {
    Small(u8),
    Flag(bool),
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Arbitrary)]
enum Parent {
    Small(u8),
    Flag(bool),
    Word(u16),
}

#[proptest]
fn project_after_embed_round_trips(child: Child) {
    prop_assert_eq!(Child::project(SubEvent::<Parent>::embed(child)), Ok(child));
}

#[proptest]
fn accepts_agrees_with_project(parent: Parent) {
    prop_assert_eq!(
        <Child as SubEvent<Parent>>::accepts(&parent),
        <Child as SubEvent<Parent>>::project(parent).is_ok()
    );
}

#[proptest]
fn rejected_event_is_returned_unchanged(word: u16) {
    prop_assert_eq!(
        <Child as SubEvent<Parent>>::project(Parent::Word(word)),
        Err(Parent::Word(word))
    );
}

#[proptest]
fn variant_round_trips(value: u8) {
    prop_assert_eq!(
        <Parent as Variant<u8>>::into_variant(Parent::from_variant(value)),
        Ok(value)
    );

    // The borrowed events must outlive the assertion macro's temporaries.
    let small = Parent::Small(value);
    let flag = Parent::Flag(true);

    prop_assert_eq!(
        <Parent as Variant<u8>>::as_variant_ref(&small),
        Some(&value)
    );
    prop_assert_eq!(<Parent as Variant<u8>>::as_variant_ref(&flag), None);
}

#[proptest]
fn from_wraps_into_matching_variant(value: u16) {
    prop_assert_eq!(Parent::from(value), Parent::Word(value));
}

#[proptest]
fn identity_projection_always_succeeds(parent: Parent) {
    prop_assert_eq!(<Parent as SubEvent<Parent>>::project(parent), Ok(parent));
}

#[test]
fn zero_variant_enum_accepts_nothing() {
    #[derive(Event)]
    enum Nothing {}

    assert!(!<Nothing as SubEvent<Parent>>::accepts(&Parent::Small(1)));
    assert!(matches!(
        <Nothing as SubEvent<Parent>>::project(Parent::Small(1)),
        Err(Parent::Small(1))
    ));
}

#[test]
fn non_clone_payload_round_trips() {
    #[derive(Debug, PartialEq)]
    struct Token(u32);

    #[derive(Event, Debug, PartialEq)]
    enum TokenEvent {
        Token(Token),
    }

    #[derive(Event, Debug, PartialEq)]
    enum Outer {
        Token(Token),
        Other(u8),
    }

    assert_eq!(
        <TokenEvent as SubEvent<Outer>>::project(Outer::Token(Token(9))),
        Ok(TokenEvent::Token(Token(9)))
    );
    assert_eq!(
        SubEvent::<Outer>::embed(TokenEvent::Token(Token(4))),
        Outer::Token(Token(4))
    );
}
