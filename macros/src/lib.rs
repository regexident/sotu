//! Procedural macros for the `sotu` crate.
//!
//! The macros are re-exported by `sotu` behind its default `macros` feature,
//! so most users never name this crate directly.

mod analyze;
mod codegen;
mod event_derive;
mod lower;
mod parse;

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

/// Generate a state machine from an impl block of state handlers.
///
/// Functions marked with `#[state]` become the leaf states of the machine. Each
/// handler produces a variant of a generated `State` enum, named with the
/// PascalCase form of the handler name, and a constructor that keeps the
/// handler's snake_case name. Variants keep the declaration order, and the
/// handler's doc comments are copied to its variant. The macro also implements
/// `sotu::blocking::IntoStateMachine` and `sotu::blocking::State`, so the storage
/// type can be turned into a running machine.
///
/// Every argument takes tokens, not a string literal:
///
/// - `initial = expression` is required and builds the state the machine
///   starts in.
/// - `context = Type` names the context handed to every handler and action. The
///   type may use the lifetimes `'ctx` and `'data` and any lifetime declared on
///   the `impl` block, and becomes `type Context<'ctx, 'data: 'ctx> = Type`. It
///   is `()` when the argument is absent. Two lifetimes are needed because
///   `&'a mut T` is invariant in `T`: with only one, a context such as
///   `Writer<'a>` holding `&'a mut Vec<&'a str>` could not be rebuilt for each
///   event inside a loop.
/// - `event_identifier = ident` renames the event input, which is `event` by
///   default.
/// - `context_identifier = ident` renames the context input, which is `context`
///   by default.
///
/// A handler takes an optional receiver, which is the shared storage, an
/// optional event input and an optional context input. Inputs are classified
/// by name only, so a renamed identifier replaces the default name, which then
/// declares local storage like any other reference input. The type behind the
/// event reference, as in `event: &Event`, is the event type of the machine. It
/// is `()` when no handler takes an event, which makes the machine usable
/// through `step`. The context input has the type `&mut Ctx<'_, '_>`, where
/// `Ctx` is the type given to `context`. State handlers, superstate handlers
/// and actions may all take the context input, and the generated code passes
/// them the context given to `handle`, `step` or `init`. A handler's attributes
/// other than the ones of `sotu` stay on the function.
///
/// Invalid arguments are reported on their own as a compile error, without the
/// impl block being examined. Once the arguments parse, the errors found in the
/// impl block are combined into one compile error, along with a missing
/// `initial`. A handler with an invalid `#[state(...)]` attribute reports that
/// attribute, and its inputs are not checked.
///
/// # Examples
///
/// ```ignore
/// use sotu::prelude::*;
///
/// #[derive(Default)]
/// struct Switch {
///     flips: u32,
/// }
///
/// struct Flip;
///
/// #[derive(Event)]
/// enum Event {
///     Flip(Flip),
/// }
///
/// #[state_machine(initial = State::off())]
/// impl Switch {
///     /// The switch is off.
///     #[state]
///     fn off(&mut self, event: &Event) -> Outcome<State> {
///         match event {
///             Event::Flip(_) => {
///                 self.flips += 1;
///
///                 Transition(State::on())
///             }
///         }
///     }
///
///     /// The switch is on.
///     #[state]
///     fn on(event: &Event) -> Outcome<State> {
///         match event {
///             Event::Flip(_) => Transition(State::off()),
///         }
///     }
/// }
///
/// let mut machine = Switch::default().state_machine();
///
/// machine.handle(Flip, &mut ());
///
/// assert_eq!(machine.flips, 1);
/// ```
///
/// Doc tests of a procedural macro crate cannot depend on `sotu`, which depends
/// on this crate, so the example above is not compiled here.
#[proc_macro_attribute]
pub fn state_machine(
    arguments: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let arguments = syn::parse_macro_input!(arguments as parse::MachineArgs);
    let item = syn::parse_macro_input!(item as syn::ItemImpl);

    analyze::analyze(arguments, item)
        .map(|model| codegen::generate(&lower::lower(&model)))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Mark a function as a state handler inside a `#[state_machine]` impl block.
///
/// The enclosing `#[state_machine]` attribute interprets the arguments and
/// removes this attribute from its output. Outside such a block this attribute
/// has no effect: it ignores its arguments and returns the item unchanged.
#[proc_macro_attribute]
pub fn state(
    _arguments: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    item
}

/// Mark a function as a superstate handler inside a `#[state_machine]` impl block.
///
/// The enclosing `#[state_machine]` attribute interprets the arguments and
/// removes this attribute from its output. Outside such a block this attribute
/// has no effect: it ignores its arguments and returns the item unchanged.
#[proc_macro_attribute]
pub fn superstate(
    _arguments: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    item
}

/// Mark an item as a mounted submachine inside a `#[state_machine]` impl block.
///
/// The enclosing `#[state_machine]` attribute interprets the arguments and
/// removes this attribute from its output. Outside such a block this attribute
/// has no effect: it ignores its arguments and returns the item unchanged.
#[proc_macro_attribute]
pub fn submachine(
    _arguments: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    item
}

/// Mark a function as an entry or exit action inside a `#[state_machine]` impl block.
///
/// The enclosing `#[state_machine]` attribute interprets the arguments and
/// removes this attribute from its output. Outside such a block this attribute
/// has no effect: it ignores its arguments and returns the item unchanged.
#[proc_macro_attribute]
pub fn action(
    _arguments: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    item
}

/// Mark a function as a submachine exit handler inside a `#[state_machine]` impl block.
///
/// The enclosing `#[state_machine]` attribute interprets the arguments and
/// removes this attribute from its output. Outside such a block this attribute
/// has no effect: it ignores its arguments and returns the item unchanged.
#[proc_macro_attribute]
pub fn exit_handler(
    _arguments: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    item
}
