use sotu::event::SubEvent;
use sotu::prelude::*;
mod inner {
    pub struct Tick;
    pub mod deeper {
        pub struct Tock;
    }
}
enum Event {
    Tick(inner::Tick),
    Tock(inner::deeper::Tock),
}
impl ::core::convert::From<inner::Tick> for Event {
    fn from(value: inner::Tick) -> Self {
        Self::Tick(value)
    }
}
impl ::sotu::event::Variant<inner::Tick> for Event {
    fn from_variant(value: inner::Tick) -> Self {
        Self::Tick(value)
    }
    fn into_variant(self) -> ::core::result::Result<inner::Tick, Self> {
        match self {
            Self::Tick(value) => ::core::result::Result::Ok(value),
            other => ::core::result::Result::Err(other),
        }
    }
    fn as_variant_ref(&self) -> ::core::option::Option<&inner::Tick> {
        match self {
            Self::Tick(value) => ::core::option::Option::Some(value),
            _ => ::core::option::Option::None,
        }
    }
}
impl ::core::convert::From<inner::deeper::Tock> for Event {
    fn from(value: inner::deeper::Tock) -> Self {
        Self::Tock(value)
    }
}
impl ::sotu::event::Variant<inner::deeper::Tock> for Event {
    fn from_variant(value: inner::deeper::Tock) -> Self {
        Self::Tock(value)
    }
    fn into_variant(self) -> ::core::result::Result<inner::deeper::Tock, Self> {
        match self {
            Self::Tock(value) => ::core::result::Result::Ok(value),
            other => ::core::result::Result::Err(other),
        }
    }
    fn as_variant_ref(&self) -> ::core::option::Option<&inner::deeper::Tock> {
        match self {
            Self::Tock(value) => ::core::option::Option::Some(value),
            _ => ::core::option::Option::None,
        }
    }
}
impl<SotuParentEvent> ::sotu::event::SubEvent<SotuParentEvent> for Event
where
    SotuParentEvent: ::sotu::event::Variant<inner::Tick>,
    SotuParentEvent: ::sotu::event::Variant<inner::deeper::Tock>,
{
    fn accepts(parent: &SotuParentEvent) -> bool {
        <SotuParentEvent as ::sotu::event::Variant<inner::Tick>>::as_variant_ref(parent)
            .is_some()
            || <SotuParentEvent as ::sotu::event::Variant<
                inner::deeper::Tock,
            >>::as_variant_ref(parent)
                .is_some()
    }
    fn project(
        parent: SotuParentEvent,
    ) -> ::core::result::Result<Self, SotuParentEvent> {
        let parent = match <SotuParentEvent as ::sotu::event::Variant<
            inner::Tick,
        >>::into_variant(parent) {
            ::core::result::Result::Ok(value) => {
                return ::core::result::Result::Ok(Self::Tick(value));
            }
            ::core::result::Result::Err(parent) => parent,
        };
        let parent = match <SotuParentEvent as ::sotu::event::Variant<
            inner::deeper::Tock,
        >>::into_variant(parent) {
            ::core::result::Result::Ok(value) => {
                return ::core::result::Result::Ok(Self::Tock(value));
            }
            ::core::result::Result::Err(parent) => parent,
        };
        ::core::result::Result::Err(parent)
    }
    fn embed(self) -> SotuParentEvent {
        match self {
            Self::Tick(value) => {
                <SotuParentEvent as ::sotu::event::Variant<
                    inner::Tick,
                >>::from_variant(value)
            }
            Self::Tock(value) => {
                <SotuParentEvent as ::sotu::event::Variant<
                    inner::deeper::Tock,
                >>::from_variant(value)
            }
        }
    }
}
fn requires<C: SubEvent<P>, P>() {}
fn main() {
    requires::<Event, Event>();
    let _tick: Event = inner::Tick.into();
    let _tock: Event = inner::deeper::Tock.into();
}
