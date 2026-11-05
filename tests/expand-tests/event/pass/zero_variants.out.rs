use sotu::event::SubEvent;
use sotu::prelude::*;
enum Event {}
impl<SotuParentEvent> ::sotu::event::SubEvent<SotuParentEvent> for Event {
    fn accepts(_parent: &SotuParentEvent) -> bool {
        false
    }
    fn project(
        parent: SotuParentEvent,
    ) -> ::core::result::Result<Self, SotuParentEvent> {
        ::core::result::Result::Err(parent)
    }
    fn embed(self) -> SotuParentEvent {
        match self {}
    }
}
enum Left {
    Number(u8),
}
impl ::core::convert::From<u8> for Left {
    fn from(value: u8) -> Self {
        Self::Number(value)
    }
}
impl ::sotu::event::Variant<u8> for Left {
    fn from_variant(value: u8) -> Self {
        Self::Number(value)
    }
    fn into_variant(self) -> ::core::result::Result<u8, Self> {
        match self {
            Self::Number(value) => ::core::result::Result::Ok(value),
        }
    }
    fn as_variant_ref(&self) -> ::core::option::Option<&u8> {
        match self {
            Self::Number(value) => ::core::option::Option::Some(value),
        }
    }
}
impl<SotuParentEvent> ::sotu::event::SubEvent<SotuParentEvent> for Left
where
    SotuParentEvent: ::sotu::event::Variant<u8>,
{
    fn accepts(parent: &SotuParentEvent) -> bool {
        <SotuParentEvent as ::sotu::event::Variant<u8>>::as_variant_ref(parent).is_some()
    }
    fn project(
        parent: SotuParentEvent,
    ) -> ::core::result::Result<Self, SotuParentEvent> {
        let parent = match <SotuParentEvent as ::sotu::event::Variant<
            u8,
        >>::into_variant(parent) {
            ::core::result::Result::Ok(value) => {
                return ::core::result::Result::Ok(Self::Number(value));
            }
            ::core::result::Result::Err(parent) => parent,
        };
        ::core::result::Result::Err(parent)
    }
    fn embed(self) -> SotuParentEvent {
        match self {
            Self::Number(value) => {
                <SotuParentEvent as ::sotu::event::Variant<u8>>::from_variant(value)
            }
        }
    }
}
enum Right {
    Flag(bool),
}
impl ::core::convert::From<bool> for Right {
    fn from(value: bool) -> Self {
        Self::Flag(value)
    }
}
impl ::sotu::event::Variant<bool> for Right {
    fn from_variant(value: bool) -> Self {
        Self::Flag(value)
    }
    fn into_variant(self) -> ::core::result::Result<bool, Self> {
        match self {
            Self::Flag(value) => ::core::result::Result::Ok(value),
        }
    }
    fn as_variant_ref(&self) -> ::core::option::Option<&bool> {
        match self {
            Self::Flag(value) => ::core::option::Option::Some(value),
        }
    }
}
impl<SotuParentEvent> ::sotu::event::SubEvent<SotuParentEvent> for Right
where
    SotuParentEvent: ::sotu::event::Variant<bool>,
{
    fn accepts(parent: &SotuParentEvent) -> bool {
        <SotuParentEvent as ::sotu::event::Variant<bool>>::as_variant_ref(parent)
            .is_some()
    }
    fn project(
        parent: SotuParentEvent,
    ) -> ::core::result::Result<Self, SotuParentEvent> {
        let parent = match <SotuParentEvent as ::sotu::event::Variant<
            bool,
        >>::into_variant(parent) {
            ::core::result::Result::Ok(value) => {
                return ::core::result::Result::Ok(Self::Flag(value));
            }
            ::core::result::Result::Err(parent) => parent,
        };
        ::core::result::Result::Err(parent)
    }
    fn embed(self) -> SotuParentEvent {
        match self {
            Self::Flag(value) => {
                <SotuParentEvent as ::sotu::event::Variant<bool>>::from_variant(value)
            }
        }
    }
}
fn requires<C: SubEvent<P>, P>() {}
fn main() {
    requires::<Event, Left>();
    requires::<Event, Right>();
    requires::<Event, Event>();
    let _number: Left = 1u8.into();
    let _flag: Right = true.into();
}
