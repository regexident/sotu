use std::cell::Cell;

use sotu::private::{DefaultInstance, MountField};
use test_strategy::proptest;

/// Value whose drop is observable through a shared flag.
struct Tracked<'flag> {
    dropped: &'flag Cell<bool>,
}

impl Drop for Tracked<'_> {
    fn drop(&mut self) {
        self.dropped.set(true);
    }
}

#[test]
fn persistent_field_is_always_present() {
    let mut value = 5u8;

    assert_eq!(MountField::<u8>::get(&value), Some(&5));

    let borrowed = MountField::<u8>::get_mut(&mut value);

    assert_eq!(borrowed.as_deref(), Some(&5));

    if let Some(child) = borrowed {
        *child = 6;
    }

    assert_eq!(value, 6);
}

#[test]
fn persistent_field_is_never_released() {
    let mut value = 5u8;

    MountField::<u8>::release(&mut value);

    assert_eq!(value, 5);
}

#[test]
fn instance_field_borrows_when_present() {
    let mut present = Some(3u8);
    let mut absent = None::<u8>;

    assert_eq!(MountField::<u8>::get(&present), Some(&3));
    assert_eq!(MountField::<u8>::get(&absent), None);

    if let Some(child) = MountField::<u8>::get_mut(&mut present) {
        *child = 4;
    }

    assert_eq!(present, Some(4));
    assert_eq!(MountField::<u8>::get_mut(&mut absent), None);
}

#[test]
fn instance_field_release_drops_instance() {
    let dropped = Cell::new(false);
    let mut field = Some(Tracked { dropped: &dropped });

    MountField::<Tracked<'_>>::release(&mut field);

    assert!(dropped.get());
    assert!(field.is_none());
}

#[proptest]
fn default_fills_only_empty_instance_field(existing: Option<u8>) {
    let mut field = existing;

    DefaultInstance::<u8>::fill_default(&mut field);

    proptest::prop_assert_eq!(field, Some(existing.unwrap_or(0)));
}

#[test]
fn persistent_field_is_never_filled() {
    let mut value = 7u8;

    DefaultInstance::<u8>::fill_default(&mut value);

    assert_eq!(value, 7);
}
