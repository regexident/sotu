use sotu::prelude::*;
struct Tick;
type Alias = Tick;
enum Event {
    A(Tick),
    B(Alias),
}
impl ::core::convert::From<Tick> for Event {
    fn from(value: Tick) -> Self {
        Self::A(value)
    }
}
impl ::sotu::event::Variant<Tick> for Event {
    fn from_variant(value: Tick) -> Self {
        Self::A(value)
    }
    fn into_variant(self) -> ::core::result::Result<Tick, Self> {
        match self {
            Self::A(value) => ::core::result::Result::Ok(value),
            other => ::core::result::Result::Err(other),
        }
    }
    fn as_variant_ref(&self) -> ::core::option::Option<&Tick> {
        match self {
            Self::A(value) => ::core::option::Option::Some(value),
            _ => ::core::option::Option::None,
        }
    }
}
impl ::core::convert::From<Alias> for Event {
    fn from(value: Alias) -> Self {
        Self::B(value)
    }
}
impl ::sotu::event::Variant<Alias> for Event {
    fn from_variant(value: Alias) -> Self {
        Self::B(value)
    }
    fn into_variant(self) -> ::core::result::Result<Alias, Self> {
        match self {
            Self::B(value) => ::core::result::Result::Ok(value),
            other => ::core::result::Result::Err(other),
        }
    }
    fn as_variant_ref(&self) -> ::core::option::Option<&Alias> {
        match self {
            Self::B(value) => ::core::option::Option::Some(value),
            _ => ::core::option::Option::None,
        }
    }
}
impl<SotuParentEvent> ::sotu::event::SubEvent<SotuParentEvent> for Event
where
    SotuParentEvent: ::sotu::event::Variant<Tick>,
    SotuParentEvent: ::sotu::event::Variant<Alias>,
{
    fn accepts(parent: &SotuParentEvent) -> bool {
        <SotuParentEvent as ::sotu::event::Variant<Tick>>::as_variant_ref(parent)
            .is_some()
            || <SotuParentEvent as ::sotu::event::Variant<Alias>>::as_variant_ref(parent)
                .is_some()
    }
    fn project(
        parent: SotuParentEvent,
    ) -> ::core::result::Result<Self, SotuParentEvent> {
        let parent = match <SotuParentEvent as ::sotu::event::Variant<
            Tick,
        >>::into_variant(parent) {
            ::core::result::Result::Ok(value) => {
                return ::core::result::Result::Ok(Self::A(value));
            }
            ::core::result::Result::Err(parent) => parent,
        };
        let parent = match <SotuParentEvent as ::sotu::event::Variant<
            Alias,
        >>::into_variant(parent) {
            ::core::result::Result::Ok(value) => {
                return ::core::result::Result::Ok(Self::B(value));
            }
            ::core::result::Result::Err(parent) => parent,
        };
        ::core::result::Result::Err(parent)
    }
    fn embed(self) -> SotuParentEvent {
        match self {
            Self::A(value) => {
                <SotuParentEvent as ::sotu::event::Variant<Tick>>::from_variant(value)
            }
            Self::B(value) => {
                <SotuParentEvent as ::sotu::event::Variant<Alias>>::from_variant(value)
            }
        }
    }
}
fn main() {}
