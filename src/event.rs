//! Event projection between parent and child machines.
//!
//! Every machine accepts exactly one event type: an enum whose variants each
//! hold a single value. The type of that value identifies the event, so two
//! variants of the same enum never hold the same type. A parent machine
//! accepts a child's events by listing the same types in its own event enum.
//!
//! [`Variant`] wraps, unwraps and peeks at the value held for one type, and
//! [`SubEvent`] projects a parent event into a child event and embeds it
//! back. Both move events instead of cloning them, so event types need not
//! implement `Clone`.
//!
//! The unit type `()` is the tick event. Its only projection is
//! `(): SubEvent<()>`, and [`unit_context`] maps any parent context to the
//! child context `()`.

// MARK: Variant

/// Wrap, unwrap and peek at the value of type `T` held by an event enum.
///
/// An event enum implements `Variant<T>` once for every type `T` that one of
/// its variants holds. `into_variant(from_variant(value))` returns
/// `Ok(value)`, and [`as_variant_ref`](Variant::as_variant_ref) returns
/// `Some` exactly when [`into_variant`](Variant::into_variant) would return
/// `Ok`. Violating these properties is a logic error.
///
/// # Examples
///
/// ```
/// use sotu::event::Variant;
///
/// enum Pair {
///     Number(u8),
///     Flag(bool),
/// }
///
/// impl Variant<u8> for Pair {
///     fn from_variant(value: u8) -> Self {
///         Self::Number(value)
///     }
///
///     fn into_variant(self) -> Result<u8, Self> {
///         match self {
///             Self::Number(value) => Ok(value),
///             other => Err(other),
///         }
///     }
///
///     fn as_variant_ref(&self) -> Option<&u8> {
///         match self {
///             Self::Number(value) => Some(value),
///             _ => None,
///         }
///     }
/// }
///
/// impl Variant<bool> for Pair {
///     fn from_variant(value: bool) -> Self {
///         Self::Flag(value)
///     }
///
///     fn into_variant(self) -> Result<bool, Self> {
///         match self {
///             Self::Flag(value) => Ok(value),
///             other => Err(other),
///         }
///     }
///
///     fn as_variant_ref(&self) -> Option<&bool> {
///         match self {
///             Self::Flag(value) => Some(value),
///             _ => None,
///         }
///     }
/// }
///
/// let pair = <Pair as Variant<u8>>::from_variant(5);
///
/// assert_eq!(<Pair as Variant<u8>>::as_variant_ref(&pair), Some(&5));
/// assert_eq!(<Pair as Variant<bool>>::as_variant_ref(&pair), None);
/// assert!(matches!(<Pair as Variant<bool>>::into_variant(pair), Err(Pair::Number(5))));
///
/// let pair = <Pair as Variant<u8>>::from_variant(5);
///
/// assert!(matches!(<Pair as Variant<u8>>::into_variant(pair), Ok(5)));
/// ```
#[diagnostic::on_unimplemented(
    message = "`{Self}` has no variant holding event `{T}`",
    note = "add a variant holding `{T}` to `{Self}`"
)]
pub trait Variant<T>: Sized {
    /// Wrap `value` in the variant of `Self` that holds a `T`.
    fn from_variant(value: T) -> Self;

    /// Unwrap the held `T` by move, or give `self` back if it holds another type.
    fn into_variant(self) -> Result<T, Self>;

    /// Borrow the held `T`, or return `None` if `self` holds another type.
    fn as_variant_ref(&self) -> Option<&T>;
}

// MARK: SubEvent

/// Project a parent event `P` into the child event `Self` and embed it back.
///
/// The child event type lists a subset of the parent's event types. The
/// following laws relate the three methods, and violating them is a logic
/// error:
///
/// - `accepts(&p)` is `true` exactly when `project(p)` returns `Ok`.
/// - `project(p)` returning `Err(q)` hands back `q == p` unchanged.
/// - `project(embed(e))` returns `Ok(e)`.
///
/// When a projection fails at a mount boundary, `sotu` treats the child as
/// having deferred to its mount node and does not panic.
///
/// # Examples
///
/// ```
/// use sotu::event::SubEvent;
///
/// #[derive(Debug, PartialEq)]
/// enum Pair {
///     Number(u8),
///     Flag(bool),
/// }
///
/// #[derive(Debug, PartialEq)]
/// enum OnlyNumber {
///     Number(u8),
/// }
///
/// impl SubEvent<Pair> for OnlyNumber {
///     fn accepts(parent: &Pair) -> bool {
///         matches!(parent, Pair::Number(_))
///     }
///
///     fn project(parent: Pair) -> Result<Self, Pair> {
///         match parent {
///             Pair::Number(value) => Ok(Self::Number(value)),
///             other => Err(other),
///         }
///     }
///
///     fn embed(self) -> Pair {
///         match self {
///             Self::Number(value) => Pair::Number(value),
///         }
///     }
/// }
///
/// assert!(OnlyNumber::accepts(&Pair::Number(3)));
/// assert!(!OnlyNumber::accepts(&Pair::Flag(true)));
/// assert_eq!(OnlyNumber::project(Pair::Number(3)), Ok(OnlyNumber::Number(3)));
/// assert_eq!(OnlyNumber::project(Pair::Flag(true)), Err(Pair::Flag(true)));
/// assert_eq!(OnlyNumber::Number(3).embed(), Pair::Number(3));
/// ```
#[diagnostic::on_unimplemented(
    message = "event `{Self}` cannot be projected from `{P}`",
    note = "every event type of the child must appear in the parent's event enum"
)]
pub trait SubEvent<P>: Sized {
    /// Return `true` if `parent` projects into `Self`.
    fn accepts(parent: &P) -> bool;

    /// Project `parent` into `Self` by move, or give `parent` back unchanged.
    fn project(parent: P) -> Result<Self, P>;

    /// Embed `self` into the parent event type.
    fn embed(self) -> P;
}

impl SubEvent<()> for () {
    fn accepts(_parent: &()) -> bool {
        true
    }

    fn project(parent: ()) -> Result<(), ()> {
        Ok(parent)
    }

    fn embed(self) {}
}

// MARK: Contexts

/// Map any parent context to the child context `()`.
///
/// This is the context mapping of children that use no context. It accepts
/// any `&mut T` and leaves it untouched.
///
/// # Examples
///
/// ```
/// use sotu::event::unit_context;
///
/// let mut context = 42u32;
/// let mapped: () = unit_context(&mut context);
///
/// assert_eq!(mapped, ());
/// assert_eq!(context, 42);
/// ```
pub fn unit_context<T>(_context: &mut T) {}
