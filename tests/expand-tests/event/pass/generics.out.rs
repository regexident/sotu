use sotu::event::{SubEvent, Variant};
use sotu::prelude::*;
struct Wrapper<T>(T);
enum Event<'a, T: Copy> {
    Borrowed(&'a str),
    Wrapped(Wrapper<T>),
}
impl<'a, T: Copy> ::core::convert::From<&'a str> for Event<'a, T> {
    fn from(value: &'a str) -> Self {
        Self::Borrowed(value)
    }
}
impl<'a, T: Copy> ::sotu::event::Variant<&'a str> for Event<'a, T> {
    fn from_variant(value: &'a str) -> Self {
        Self::Borrowed(value)
    }
    fn into_variant(self) -> ::core::result::Result<&'a str, Self> {
        match self {
            Self::Borrowed(value) => ::core::result::Result::Ok(value),
            other => ::core::result::Result::Err(other),
        }
    }
    fn as_variant_ref(&self) -> ::core::option::Option<&&'a str> {
        match self {
            Self::Borrowed(value) => ::core::option::Option::Some(value),
            _ => ::core::option::Option::None,
        }
    }
}
impl<'a, T: Copy> ::core::convert::From<Wrapper<T>> for Event<'a, T> {
    fn from(value: Wrapper<T>) -> Self {
        Self::Wrapped(value)
    }
}
impl<'a, T: Copy> ::sotu::event::Variant<Wrapper<T>> for Event<'a, T> {
    fn from_variant(value: Wrapper<T>) -> Self {
        Self::Wrapped(value)
    }
    fn into_variant(self) -> ::core::result::Result<Wrapper<T>, Self> {
        match self {
            Self::Wrapped(value) => ::core::result::Result::Ok(value),
            other => ::core::result::Result::Err(other),
        }
    }
    fn as_variant_ref(&self) -> ::core::option::Option<&Wrapper<T>> {
        match self {
            Self::Wrapped(value) => ::core::option::Option::Some(value),
            _ => ::core::option::Option::None,
        }
    }
}
impl<'a, SotuParentEvent, T: Copy> ::sotu::event::SubEvent<SotuParentEvent>
for Event<'a, T>
where
    SotuParentEvent: ::sotu::event::Variant<&'a str>,
    SotuParentEvent: ::sotu::event::Variant<Wrapper<T>>,
{
    fn accepts(parent: &SotuParentEvent) -> bool {
        <SotuParentEvent as ::sotu::event::Variant<&'a str>>::as_variant_ref(parent)
            .is_some()
            || <SotuParentEvent as ::sotu::event::Variant<
                Wrapper<T>,
            >>::as_variant_ref(parent)
                .is_some()
    }
    fn project(
        parent: SotuParentEvent,
    ) -> ::core::result::Result<Self, SotuParentEvent> {
        let parent = match <SotuParentEvent as ::sotu::event::Variant<
            &'a str,
        >>::into_variant(parent) {
            ::core::result::Result::Ok(value) => {
                return ::core::result::Result::Ok(Self::Borrowed(value));
            }
            ::core::result::Result::Err(parent) => parent,
        };
        let parent = match <SotuParentEvent as ::sotu::event::Variant<
            Wrapper<T>,
        >>::into_variant(parent) {
            ::core::result::Result::Ok(value) => {
                return ::core::result::Result::Ok(Self::Wrapped(value));
            }
            ::core::result::Result::Err(parent) => parent,
        };
        ::core::result::Result::Err(parent)
    }
    fn embed(self) -> SotuParentEvent {
        match self {
            Self::Borrowed(value) => {
                <SotuParentEvent as ::sotu::event::Variant<&'a str>>::from_variant(value)
            }
            Self::Wrapped(value) => {
                <SotuParentEvent as ::sotu::event::Variant<
                    Wrapper<T>,
                >>::from_variant(value)
            }
        }
    }
}
fn requires<C: SubEvent<P>, P>() {}
fn main() {
    requires::<Event<'_, u8>, Event<'_, u8>>();
    let event: Event<'_, u8> = Event::from("text");
    match (&<Event<'_, u8> as Variant<&str>>::as_variant_ref(&event), &Some(&"text")) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    let event: Event<'_, u8> = Wrapper(7u8).into();
    let Ok(Wrapper(inner)) = <Event<'_, u8> as Variant<Wrapper<u8>>>::into_variant(event)
    else {
        return;
    };
    match (&inner, &7) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
}
