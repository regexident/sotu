use sotu::event::{SubEvent, Variant};
use sotu::prelude::*;

struct Wrapper<T>(T);

#[derive(Event)]
enum Event<'a, T: Copy> {
    Borrowed(&'a str),
    Wrapped(Wrapper<T>),
}

fn requires<C: SubEvent<P>, P>() {}

fn main() {
    requires::<Event<'_, u8>, Event<'_, u8>>();

    let event: Event<'_, u8> = Event::from("text");

    assert_eq!(<Event<'_, u8> as Variant<&str>>::as_variant_ref(&event), Some(&"text"));

    let event: Event<'_, u8> = Wrapper(7u8).into();

    let Ok(Wrapper(inner)) = <Event<'_, u8> as Variant<Wrapper<u8>>>::into_variant(event) else {
        return;
    };

    assert_eq!(inner, 7);
}
