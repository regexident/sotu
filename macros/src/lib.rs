//! Procedural macros for the `sotu` crate.
//!
//! The macros are re-exported by `sotu` behind its default `macros` feature,
//! so most users never name this crate directly.

/// Derive the `From`, `Variant` and `SubEvent` implementations of an event enum.
#[proc_macro_derive(Event)]
pub fn derive_event(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    input
}
