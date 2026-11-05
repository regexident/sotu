//! Procedural macros for the `sotu` crate.
//!
//! The macros are re-exported by `sotu` behind its default `macros` feature,
//! so most users never name this crate directly.

mod event_derive;

/// Derive the `From`, `Variant` and `SubEvent` implementations of an event enum.
///
/// Every variant of the enum must be a tuple variant holding exactly one
/// value, and no two variants may hold the same type. For each variant
/// `V(T)` the derive implements [`From<T>`] and `sotu::event::Variant<T>`
/// for the enum. It also implements `sotu::event::SubEvent<P>` for every
/// parent event `P` that implements `Variant<T>` for all of the enum's variant
/// types, which makes the enum a child event of any such parent. An enum
/// without variants is a child event of every parent and accepts none of
/// their events.
///
/// # Examples
///
/// ```ignore
/// use sotu::Event;
///
/// struct Tick;
/// struct Reset;
///
/// #[derive(Event)]
/// enum Command {
///     Tick(Tick),
///     Reset(Reset),
/// }
/// ```
///
/// Doc tests of a procedural macro crate cannot depend on `sotu`, which
/// depends on this crate, so the example above is not compiled here. The
/// expansion is covered by the tests of the `sotu::Event` re-export.
#[proc_macro_derive(Event)]
pub fn derive_event(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    event_derive::expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
