use sotu::event::Variant;
use sotu::prelude::*;

struct Tick;

#[derive(Event)]
enum Event {
    Tick(Tick),
}

fn main() {
    let event: Event = Tick.into();

    assert!(<Event as Variant<Tick>>::as_variant_ref(&event).is_some());
}
