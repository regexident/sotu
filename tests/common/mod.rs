// Each test crate uses a different subset of the shared fixtures, so unused
// items are expected in any single crate.
#![allow(dead_code)]

pub mod bare;
pub mod tree;

/// Compile only when `$type` does not implement `$trait`.
///
/// The path `<$type as AmbiguousIfImpl<_>>` asks the compiler to infer `A`. The
/// impl for `A = ()` always applies, and the impl for `A = Marker` applies only
/// when `$type: $trait` holds. With one applicable impl, inference picks `()`.
/// With two, inference has no unique answer and compilation fails.
#[allow(unused_macros)]
macro_rules! assert_not_impl {
    ($type:ty: $trait:path) => {
        const _: fn() = || {
            struct Marker;

            trait AmbiguousIfImpl<A> {
                fn some_item() {}
            }

            impl<T: ?Sized> AmbiguousIfImpl<()> for T {}

            impl<T: ?Sized + $trait> AmbiguousIfImpl<Marker> for T {}

            let _ = <$type as AmbiguousIfImpl<_>>::some_item;
        };
    };
}

#[allow(unused_imports)]
pub(crate) use assert_not_impl;
