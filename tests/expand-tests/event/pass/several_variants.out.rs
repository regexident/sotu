use sotu::event::SubEvent;
use sotu::prelude::*;
struct Start;
struct Stop;
struct Reset;
enum Parent {
    Start(Start),
    Stop(Stop),
    Reset(Reset),
}
impl ::core::convert::From<Start> for Parent {
    fn from(value: Start) -> Self {
        Self::Start(value)
    }
}
impl ::sotu::event::Variant<Start> for Parent {
    fn from_variant(value: Start) -> Self {
        Self::Start(value)
    }
    fn into_variant(self) -> ::core::result::Result<Start, Self> {
        match self {
            Self::Start(value) => ::core::result::Result::Ok(value),
            other => ::core::result::Result::Err(other),
        }
    }
    fn as_variant_ref(&self) -> ::core::option::Option<&Start> {
        match self {
            Self::Start(value) => ::core::option::Option::Some(value),
            _ => ::core::option::Option::None,
        }
    }
}
impl ::core::convert::From<Stop> for Parent {
    fn from(value: Stop) -> Self {
        Self::Stop(value)
    }
}
impl ::sotu::event::Variant<Stop> for Parent {
    fn from_variant(value: Stop) -> Self {
        Self::Stop(value)
    }
    fn into_variant(self) -> ::core::result::Result<Stop, Self> {
        match self {
            Self::Stop(value) => ::core::result::Result::Ok(value),
            other => ::core::result::Result::Err(other),
        }
    }
    fn as_variant_ref(&self) -> ::core::option::Option<&Stop> {
        match self {
            Self::Stop(value) => ::core::option::Option::Some(value),
            _ => ::core::option::Option::None,
        }
    }
}
impl ::core::convert::From<Reset> for Parent {
    fn from(value: Reset) -> Self {
        Self::Reset(value)
    }
}
impl ::sotu::event::Variant<Reset> for Parent {
    fn from_variant(value: Reset) -> Self {
        Self::Reset(value)
    }
    fn into_variant(self) -> ::core::result::Result<Reset, Self> {
        match self {
            Self::Reset(value) => ::core::result::Result::Ok(value),
            other => ::core::result::Result::Err(other),
        }
    }
    fn as_variant_ref(&self) -> ::core::option::Option<&Reset> {
        match self {
            Self::Reset(value) => ::core::option::Option::Some(value),
            _ => ::core::option::Option::None,
        }
    }
}
impl<SotuParentEvent> ::sotu::event::SubEvent<SotuParentEvent> for Parent
where
    SotuParentEvent: ::sotu::event::Variant<Start>,
    SotuParentEvent: ::sotu::event::Variant<Stop>,
    SotuParentEvent: ::sotu::event::Variant<Reset>,
{
    fn accepts(parent: &SotuParentEvent) -> bool {
        <SotuParentEvent as ::sotu::event::Variant<Start>>::as_variant_ref(parent)
            .is_some()
            || <SotuParentEvent as ::sotu::event::Variant<Stop>>::as_variant_ref(parent)
                .is_some()
            || <SotuParentEvent as ::sotu::event::Variant<Reset>>::as_variant_ref(parent)
                .is_some()
    }
    fn project(
        parent: SotuParentEvent,
    ) -> ::core::result::Result<Self, SotuParentEvent> {
        let parent = match <SotuParentEvent as ::sotu::event::Variant<
            Start,
        >>::into_variant(parent) {
            ::core::result::Result::Ok(value) => {
                return ::core::result::Result::Ok(Self::Start(value));
            }
            ::core::result::Result::Err(parent) => parent,
        };
        let parent = match <SotuParentEvent as ::sotu::event::Variant<
            Stop,
        >>::into_variant(parent) {
            ::core::result::Result::Ok(value) => {
                return ::core::result::Result::Ok(Self::Stop(value));
            }
            ::core::result::Result::Err(parent) => parent,
        };
        let parent = match <SotuParentEvent as ::sotu::event::Variant<
            Reset,
        >>::into_variant(parent) {
            ::core::result::Result::Ok(value) => {
                return ::core::result::Result::Ok(Self::Reset(value));
            }
            ::core::result::Result::Err(parent) => parent,
        };
        ::core::result::Result::Err(parent)
    }
    fn embed(self) -> SotuParentEvent {
        match self {
            Self::Start(value) => {
                <SotuParentEvent as ::sotu::event::Variant<Start>>::from_variant(value)
            }
            Self::Stop(value) => {
                <SotuParentEvent as ::sotu::event::Variant<Stop>>::from_variant(value)
            }
            Self::Reset(value) => {
                <SotuParentEvent as ::sotu::event::Variant<Reset>>::from_variant(value)
            }
        }
    }
}
enum Child {
    Start(Start),
    Stop(Stop),
}
impl ::core::convert::From<Start> for Child {
    fn from(value: Start) -> Self {
        Self::Start(value)
    }
}
impl ::sotu::event::Variant<Start> for Child {
    fn from_variant(value: Start) -> Self {
        Self::Start(value)
    }
    fn into_variant(self) -> ::core::result::Result<Start, Self> {
        match self {
            Self::Start(value) => ::core::result::Result::Ok(value),
            other => ::core::result::Result::Err(other),
        }
    }
    fn as_variant_ref(&self) -> ::core::option::Option<&Start> {
        match self {
            Self::Start(value) => ::core::option::Option::Some(value),
            _ => ::core::option::Option::None,
        }
    }
}
impl ::core::convert::From<Stop> for Child {
    fn from(value: Stop) -> Self {
        Self::Stop(value)
    }
}
impl ::sotu::event::Variant<Stop> for Child {
    fn from_variant(value: Stop) -> Self {
        Self::Stop(value)
    }
    fn into_variant(self) -> ::core::result::Result<Stop, Self> {
        match self {
            Self::Stop(value) => ::core::result::Result::Ok(value),
            other => ::core::result::Result::Err(other),
        }
    }
    fn as_variant_ref(&self) -> ::core::option::Option<&Stop> {
        match self {
            Self::Stop(value) => ::core::option::Option::Some(value),
            _ => ::core::option::Option::None,
        }
    }
}
impl<SotuParentEvent> ::sotu::event::SubEvent<SotuParentEvent> for Child
where
    SotuParentEvent: ::sotu::event::Variant<Start>,
    SotuParentEvent: ::sotu::event::Variant<Stop>,
{
    fn accepts(parent: &SotuParentEvent) -> bool {
        <SotuParentEvent as ::sotu::event::Variant<Start>>::as_variant_ref(parent)
            .is_some()
            || <SotuParentEvent as ::sotu::event::Variant<Stop>>::as_variant_ref(parent)
                .is_some()
    }
    fn project(
        parent: SotuParentEvent,
    ) -> ::core::result::Result<Self, SotuParentEvent> {
        let parent = match <SotuParentEvent as ::sotu::event::Variant<
            Start,
        >>::into_variant(parent) {
            ::core::result::Result::Ok(value) => {
                return ::core::result::Result::Ok(Self::Start(value));
            }
            ::core::result::Result::Err(parent) => parent,
        };
        let parent = match <SotuParentEvent as ::sotu::event::Variant<
            Stop,
        >>::into_variant(parent) {
            ::core::result::Result::Ok(value) => {
                return ::core::result::Result::Ok(Self::Stop(value));
            }
            ::core::result::Result::Err(parent) => parent,
        };
        ::core::result::Result::Err(parent)
    }
    fn embed(self) -> SotuParentEvent {
        match self {
            Self::Start(value) => {
                <SotuParentEvent as ::sotu::event::Variant<Start>>::from_variant(value)
            }
            Self::Stop(value) => {
                <SotuParentEvent as ::sotu::event::Variant<Stop>>::from_variant(value)
            }
        }
    }
}
fn requires<C: SubEvent<P>, P>() {}
fn main() {
    requires::<Child, Parent>();
    requires::<Parent, Parent>();
    let _start: Parent = Start.into();
    let _stop: Parent = Stop.into();
    let _reset: Parent = Reset.into();
}
