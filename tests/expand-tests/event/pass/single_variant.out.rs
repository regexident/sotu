use sotu::event::Variant;
use sotu::prelude::*;
struct Tick;
enum Event {
    Tick(Tick),
}
impl ::core::convert::From<Tick> for Event {
    fn from(value: Tick) -> Self {
        Self::Tick(value)
    }
}
impl ::sotu::event::Variant<Tick> for Event {
    fn from_variant(value: Tick) -> Self {
        Self::Tick(value)
    }
    fn into_variant(self) -> ::core::result::Result<Tick, Self> {
        match self {
            Self::Tick(value) => ::core::result::Result::Ok(value),
        }
    }
    fn as_variant_ref(&self) -> ::core::option::Option<&Tick> {
        match self {
            Self::Tick(value) => ::core::option::Option::Some(value),
        }
    }
}
impl<SotuParentEvent> ::sotu::event::SubEvent<SotuParentEvent> for Event
where
    SotuParentEvent: ::sotu::event::Variant<Tick>,
{
    fn accepts(parent: &SotuParentEvent) -> bool {
        <SotuParentEvent as ::sotu::event::Variant<Tick>>::as_variant_ref(parent)
            .is_some()
    }
    fn project(
        parent: SotuParentEvent,
    ) -> ::core::result::Result<Self, SotuParentEvent> {
        let parent = match <SotuParentEvent as ::sotu::event::Variant<
            Tick,
        >>::into_variant(parent) {
            ::core::result::Result::Ok(value) => {
                return ::core::result::Result::Ok(Self::Tick(value));
            }
            ::core::result::Result::Err(parent) => parent,
        };
        ::core::result::Result::Err(parent)
    }
    fn embed(self) -> SotuParentEvent {
        match self {
            Self::Tick(value) => {
                <SotuParentEvent as ::sotu::event::Variant<Tick>>::from_variant(value)
            }
        }
    }
}
fn main() {
    let event: Event = Tick.into();
    if !<Event as Variant<Tick>>::as_variant_ref(&event).is_some() {
        ::core::panicking::panic(
            "assertion failed: <Event as Variant<Tick>>::as_variant_ref(&event).is_some()",
        )
    }
}
