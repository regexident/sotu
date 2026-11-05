use sotu::prelude::*;
mod common {
    pub struct Tick;
}
enum Event {
    A(common::Tick),
    B(common::Tick),
}
impl ::core::convert::From<common::Tick> for Event {
    fn from(value: common::Tick) -> Self {
        Self::A(value)
    }
}
impl ::sotu::event::Variant<common::Tick> for Event {
    fn from_variant(value: common::Tick) -> Self {
        Self::A(value)
    }
    fn into_variant(self) -> ::core::result::Result<common::Tick, Self> {
        match self {
            Self::A(value) => ::core::result::Result::Ok(value),
            other => ::core::result::Result::Err(other),
        }
    }
    fn as_variant_ref(&self) -> ::core::option::Option<&common::Tick> {
        match self {
            Self::A(value) => ::core::option::Option::Some(value),
            _ => ::core::option::Option::None,
        }
    }
}
impl ::core::convert::From<common::Tick> for Event {
    fn from(value: common::Tick) -> Self {
        Self::B(value)
    }
}
impl ::sotu::event::Variant<common::Tick> for Event {
    fn from_variant(value: common::Tick) -> Self {
        Self::B(value)
    }
    fn into_variant(self) -> ::core::result::Result<common::Tick, Self> {
        match self {
            Self::B(value) => ::core::result::Result::Ok(value),
            other => ::core::result::Result::Err(other),
        }
    }
    fn as_variant_ref(&self) -> ::core::option::Option<&common::Tick> {
        match self {
            Self::B(value) => ::core::option::Option::Some(value),
            _ => ::core::option::Option::None,
        }
    }
}
impl<SotuParentEvent> ::sotu::event::SubEvent<SotuParentEvent> for Event
where
    SotuParentEvent: ::sotu::event::Variant<common::Tick>,
    SotuParentEvent: ::sotu::event::Variant<common::Tick>,
{
    fn accepts(parent: &SotuParentEvent) -> bool {
        <SotuParentEvent as ::sotu::event::Variant<common::Tick>>::as_variant_ref(parent)
            .is_some()
            || <SotuParentEvent as ::sotu::event::Variant<
                common::Tick,
            >>::as_variant_ref(parent)
                .is_some()
    }
    fn project(
        parent: SotuParentEvent,
    ) -> ::core::result::Result<Self, SotuParentEvent> {
        let parent = match <SotuParentEvent as ::sotu::event::Variant<
            common::Tick,
        >>::into_variant(parent) {
            ::core::result::Result::Ok(value) => {
                return ::core::result::Result::Ok(Self::A(value));
            }
            ::core::result::Result::Err(parent) => parent,
        };
        let parent = match <SotuParentEvent as ::sotu::event::Variant<
            common::Tick,
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
                <SotuParentEvent as ::sotu::event::Variant<
                    common::Tick,
                >>::from_variant(value)
            }
            Self::B(value) => {
                <SotuParentEvent as ::sotu::event::Variant<
                    common::Tick,
                >>::from_variant(value)
            }
        }
    }
}
fn main() {}
