//! Procedural macros for the `sotu` crate.
//!
//! The macros are re-exported by `sotu` behind its default `macros` feature,
//! so most users never name this crate directly.
//!
//! [`state_machine`] turns an impl block of handlers into a complete state
//! machine, and the item attributes [`state`], [`superstate`], [`submachine`],
//! [`action`] and [`exit_handler`] mark the functions it consumes.
//! [`derive@Event`] builds the event enum that a machine and its parents share.

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
/// Functions marked with `#[state]` become the leaf states of the machine,
/// functions marked with `#[superstate]` its superstates, and functions marked
/// with `#[submachine]` the mounts of other machines. The macro implements
/// `sotu::blocking::IntoStateMachine` for the storage type, and
/// `sotu::blocking::State` and `sotu::blocking::Superstate` for the generated
/// enums, or their counterparts in `sotu::awaitable`. The extension trait
/// `IntoStateMachineExt`, which `sotu::prelude` brings into scope, then turns
/// the storage into a running machine.
///
/// Key values are written as tokens, and a value written as a string literal is
/// rejected. Each key may appear once. Invalid keys are reported on their own
/// as a compile error, without the impl block being examined. Once the keys
/// parse, the errors found in the impl block are combined into one compile
/// error, along with a missing `initial`. A function whose item attribute has
/// invalid arguments is not classified, so its inputs are not checked.
///
/// # Keys
///
/// | Key | Value | Default | Effect |
/// |-----|-------|---------|--------|
/// | `initial` | expression | none, required | Builds the state the machine starts in, such as `State::idle()`. |
/// | `context` | type | `()` | The context handed to every handler, action and hook, which becomes `type Context<'ctx, 'data: 'ctx> = Type`. The type may use `'ctx`, `'data` and any lifetime of the impl block. |
/// | `exit` | type | `::core::convert::Infallible` | The exit points through which the machine can be left, which becomes `type Exit = Type`. Handlers then return `Outcome<State, Type>`. |
/// | `event_identifier` | identifier | `event` | The name of the event input of handlers. |
/// | `context_identifier` | identifier | `context` | The name of the context input of handlers, actions and exit handlers. |
/// | `submachine_identifier` | identifier | `submachine` | The name of the input that borrows a mounted machine's storage. |
/// | `exit_identifier` | identifier | `exit` | The name of the input of an exit handler that receives the mounted machine's exit value. |
/// | `before_dispatch` | path | no hook | A function called just before each node handles an event. |
/// | `after_dispatch` | path | no hook | A function called just after each node handled an event. |
/// | `before_transition` | path | no hook | A function called just before the first exit action of a transition. |
/// | `after_transition` | path | no hook | A function called just after the last entry action of a transition. |
/// | `visibility` | visibility | `pub` | The visibility of the generated enums and their constructors, such as `pub(crate)`. |
/// | `state(...)` | list | see below | The options of the generated `State` enum. |
/// | `superstate(...)` | list | see below | The options of the generated `Superstate` enum. |
/// | `awaitable` | bare flag | absent | Makes the machine awaitable even if no function is an `async fn`. |
///
/// A dispatch hook takes `&mut self`, the node as a
/// `StateOrSuperstate<'_, State, Superstate>` and the event by reference. A
/// transition hook takes `&mut self`, the source state and the target state by
/// reference. Either may take the context as a trailing parameter, and the
/// generated code passes the context only to hooks that take it. In the node of
/// a dispatch hook, `Superstate` is `()` for a machine with neither
/// superstates nor mounts and carries a lifetime if its variants borrow local
/// storage or hold a mount. A hook that is absent keeps the no-op default of
/// the `IntoStateMachine` trait. Hooks may be methods of the impl block, named
/// as `Self::name`, or free functions.
///
/// Identifier keys classify inputs by name only, so a renamed identifier
/// replaces the default name, which then declares local storage like any other
/// reference input.
///
/// The options of `state(...)` and `superstate(...)` are:
///
/// | Option | Applies to | Default | Effect |
/// |--------|------------|---------|--------|
/// | `name = Name` | both | `State` and `Superstate` | The name of the generated enum. |
/// | `derive(Trait, ...)` | both | no derives | Traits to derive on the generated enum. |
/// | `custom` | `state` only | absent | The impl block uses an enum that the user defines, so the macro generates no enum and no constructors. The enum needs one struct variant per state handler, named after the handler in PascalCase or by its `name` key, with fields named and typed like the handler's local storage, and one tuple variant per mount that holds the child's `State`. Cannot be combined with `derive`. |
///
/// # Awaitable machines
///
/// The machine is awaitable when any `#[state]`, `#[superstate]` or
/// `#[submachine]` handler, `#[action]` or `#[exit_handler]` is an `async fn`,
/// when a hook names an `async fn` of the impl block as `name` or `Self::name`,
/// or when `awaitable` is given. An awaitable machine implements the traits of
/// `sotu::awaitable` in place of those of `sotu::blocking`, awaits its `async`
/// handlers, actions and hooks, and calls its plain ones directly. Hooks
/// defined elsewhere are not inspected, so a machine whose only `async` items
/// are such hooks must declare `awaitable`. An awaitable machine can mount
/// submachines of either mode, and a blocking machine can mount blocking
/// submachines only.
///
/// # Handlers and their inputs
///
/// The event type of the machine is the type behind the event reference, as in
/// `event: &Event`. It is `()` when no handler takes an event, which makes the
/// machine usable through `step`. The context input has the type
/// `&mut Ctx<'_, '_>`, where `Ctx` is the type given to `context`.
///
/// Every handler, action and exit handler may take the context input. Two
/// lifetimes exist because `&'a mut T` is invariant in `T`: with only one,
/// a context such as `Writer<'a>` holding `&'a mut Vec<&'a str>` could not be
/// rebuilt for each event inside a loop.
///
/// A handler may declare generics only through the impl block. Inputs are
/// plain identifiers. The event input must be a shared reference, and every
/// input other than the receiver, the event, the context and the exit value of
/// an exit handler must be a reference. A handler's
/// attributes other than the ones of `sotu` stay on the function, and its doc
/// comments are copied to its variant.
///
/// # Item attributes
///
/// [`#[state]`](macro@state) declares a leaf state. Its inputs are an optional
/// receiver (`&self` or `&mut self`, the shared storage), an optional event, an
/// optional context, and any number of local storage references. A local
/// storage input becomes a field of the state's variant and a parameter of its
/// constructor. `#[default]` or `#[default = expression]` on such an input
/// removes it from the constructor and fills it with `Default::default()` or
/// the expression.
///
/// [`#[superstate]`](macro@superstate) declares a node that groups states and
/// other superstates. Its inputs are the same as those of a state, except that
/// local storage inputs are borrowed from the active leaf, so the leaves below
/// it need fields of the same names and types. A superstate with local storage
/// cannot contain a mount, because the leaves below a mount belong to the
/// mounted machine and have no such fields.
///
/// [`#[submachine]`](macro@submachine) mounts another machine as a node. The
/// annotated function is the mount handler, which receives the events that
/// reach the mount node. Its inputs are an optional receiver, event and
/// context, and optionally the `submachine` input, a reference to the child's
/// storage. The `submachine` input and the receiver cannot be combined, because
/// the input borrows a field of the storage that the receiver borrows whole.
/// The same restriction applies to the actions and the exit handler of a mount
/// that take the `submachine` input. The mount handler takes no local storage.
///
/// [`#[action]`](macro@action) declares an entry or exit action. Its inputs are
/// an optional receiver, an optional context and references to the local
/// storage of the state or superstate that names it. The actions of a mount
/// may also take the `submachine` input.
///
/// [`#[exit_handler]`](macro@exit_handler) declares the function that answers
/// the exit value of a mounted machine with an outcome of the parent. It takes
/// the exit value by value, named by `exit_identifier`, and may take the
/// receiver, the context and the `submachine` input.
///
/// The keys of the item attributes are:
///
/// | Attribute | Key | Default | Effect |
/// |-----------|-----|---------|--------|
/// | `#[state]` | `name = Name` | PascalCase of the function name | The name of the variant. The constructor is its snake_case form. |
/// | `#[state]`, `#[superstate]`, `#[submachine]` | `superstate = handler` | top level | The `#[superstate]` or `#[submachine]` handler that encloses the node. |
/// | `#[state]`, `#[superstate]`, `#[submachine]` | `entry_action = action` | none | The `#[action]` that runs when the node is entered. |
/// | `#[state]`, `#[superstate]`, `#[submachine]` | `exit_action = action` | none | The `#[action]` that runs when the node is left. |
/// | `#[state]`, `#[superstate]` | `local_storage(field: Type, ...)` | none | Extra fields of the variant that no handler input declares. A `#[superstate]` uses the listed type as written, so it must be the reference that the superstate borrows from the leaf, such as `unit: &'sub mut char` or `unit: &'sub char`. The leaf declares the plain `unit: char` field. |
/// | `#[superstate]` | `initial = expression` | none | The state that the generated constructor `Superstate::name()` returns. |
/// | `#[superstate]`, `#[submachine]` | `name = Name` | PascalCase of the function name | The name of the variant. |
/// | `#[submachine]` | `machine = Type` | none, required | The type of the mounted machine. |
/// | `#[submachine]` | `field = ident` | none, required | The field of the storage that holds the mounted machine. |
/// | `#[submachine]` | `factory = path` | `Default::default()` | A function `fn(&mut self, &mut Context) -> Machine` that creates the machine. Only valid for an `Option<Machine>` field. |
/// | `#[submachine]` | `context = path` | the parent's context unchanged | A function from the parent's context to the child's, called for each call into the child. `sotu::event::unit_context` maps any context to `()`. |
/// | `#[submachine]` | `exit_handler = handler` | none | The `#[exit_handler]` that receives the child's exit values. Required when the child declares an exit type. |
/// | `#[action]`, `#[exit_handler]` | none | | These attributes take no arguments. |
///
/// A mount's entry action runs after the child is created and its exit action
/// runs before the child is released. The type of the `field` decides how the
/// child lives. An `Option<Machine>` field is filled when the mount is entered,
/// unless it already holds a child, and emptied when the mount is left, so the
/// child is created and dropped with the mount. A plain `Machine` field is
/// never created or dropped by a transition and keeps its storage while the
/// mount is inactive.
///
/// # Generated items
///
/// - `State`, an enum with one struct variant per `#[state]` and one tuple
///   variant per `#[submachine]`. Local storage inputs are the fields of a state
///   variant, and the variant of a mount holds the child's `State`.
/// - A constructor per state variant, named after the variant in snake_case,
///   taking the fields that have no `#[default]`. A mount's constructor takes
///   the child's state.
/// - `Superstate`, an enum with one variant per `#[superstate]` and per
///   `#[submachine]`. The variant of a mount holds a
///   `sotu::SubmachineNode`, which is `Mount` for the mount node and `Inner`
///   for a superstate of the child. The enum has a lifetime parameter if a
///   superstate borrows local storage or the machine has a mount, and the
///   superstate type is `()` if the machine has neither superstates nor mounts.
/// - For each `#[superstate]` with `initial` and for each mount, a constructor
///   `Superstate::name()` that returns the state to enter the node at. For a
///   mount it is the child's initial state.
/// - The trait implementations named above, and the implementations that let
///   the machine be mounted in another machine.
///
/// Machines without exit points have the exit type `Infallible`, and a
/// handler returns `Outcome<State>`. Handlers that return `Exit` at the root
/// of a machine terminate it.
///
/// # Examples
///
/// A machine with a superstate, local storage, actions and a hook:
///
/// ```ignore
/// use sotu::prelude::*;
///
/// #[derive(Default)]
/// struct Lamp {
///     log: Vec<&'static str>,
///     transitions: u32,
/// }
///
/// struct Toggle;
///
/// #[derive(Event)]
/// enum Event {
///     Toggle(Toggle),
/// }
///
/// #[state_machine(initial = State::off(), after_transition = Self::count)]
/// impl Lamp {
///     #[superstate(entry_action = enter_powered, exit_action = exit_powered)]
///     fn powered(event: &Event) -> Outcome<State> {
///         match event {
///             Event::Toggle(_) => Super,
///         }
///     }
///
///     #[state(superstate = powered)]
///     fn off(event: &Event) -> Outcome<State> {
///         match event {
///             Event::Toggle(_) => Transition(State::on(100)),
///         }
///     }
///
///     #[state(superstate = powered)]
///     fn on(brightness: &mut u8, event: &Event) -> Outcome<State> {
///         match event {
///             Event::Toggle(_) => Transition(State::off()),
///         }
///     }
///
///     fn count(&mut self, _source: &State, _target: &State) {
///         self.transitions += 1;
///     }
///
///     #[action]
///     fn enter_powered(&mut self) {
///         self.log.push("enter powered");
///     }
///
///     #[action]
///     fn exit_powered(&mut self) {
///         self.log.push("exit powered");
///     }
/// }
/// ```
///
/// A machine that mounts another machine, with a factory and an exit handler:
///
/// ```ignore
/// #[state_machine(initial = State::idle())]
/// impl Washer {
///     #[state]
///     fn idle(event: &Event) -> Outcome<State> {
///         match event {
///             Event::Start(_) => Transition(Superstate::loading()),
///             Event::Close(_) | Event::Tick(_) => Super,
///         }
///     }
///
///     #[submachine(
///         machine = door::Door,
///         field = door,
///         factory = Self::new_door,
///         exit_handler = door_exited
///     )]
///     fn loading(event: &Event) -> Outcome<State> {
///         match event {
///             Event::Start(_) | Event::Close(_) | Event::Tick(_) => Super,
///         }
///     }
///
///     #[exit_handler]
///     fn door_exited(exit: door::DoorExit) -> Outcome<State> {
///         match exit {
///             door::DoorExit::Closed => Transition(State::idle()),
///         }
///     }
///
///     fn new_door(&mut self, _context: &mut ()) -> door::Door {
///         door::Door::default()
///     }
/// }
/// ```
///
/// Doc tests of a procedural macro crate cannot depend on `sotu`, which depends
/// on this crate, so the examples above are not compiled here.
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
/// The handler becomes a leaf state: a variant of the generated `State` enum
/// whose fields are the handler's local storage, and a constructor of the same
/// name. The keys are `name`, `superstate`, `entry_action`, `exit_action` and
/// `local_storage(field: Type, ...)`, and the inputs are an optional receiver,
/// event and context followed by local storage references. The keys, their
/// defaults and the inputs are described in the documentation of
/// [`state_machine`](macro@state_machine).
///
/// The enclosing `#[state_machine]` attribute interprets the arguments and
/// removes this attribute from its output. Outside such a block this attribute
/// has no effect: it ignores its arguments and returns the item unchanged.
///
/// # Examples
///
/// This handler declares the variant `State::First { value, unit }` and the
/// constructor `State::first(value, unit)`. The `value` field comes from the
/// handler's input and the `unit` field from the `local_storage` key, which the
/// handler does not take as an input:
///
/// ```ignore
/// #[state(superstate = keypad, entry_action = clear_display, local_storage(unit: char))]
/// fn first(value: &mut i64, event: &Event) -> Outcome<State> {
///     match event {
///         Event::Digit(Digit(digit)) => {
///             *value = *value * 10 + i64::from(*digit);
///
///             Handled
///         }
///         Event::Clear(_) => Transition(State::first(0, ' ')),
///         Event::Plus(_) => Super,
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn state(
    _arguments: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    item
}

/// Mark a function as a superstate handler inside a `#[state_machine]` impl block.
///
/// The handler becomes a variant of the generated `Superstate` enum. It
/// receives the events that the states below it defer with `Super`, and its
/// local storage inputs are borrowed from the active leaf. The keys are `name`,
/// `superstate`, `entry_action`, `exit_action`, `initial` and
/// `local_storage(field: Type, ...)`. With `initial = expression`, the macro
/// generates a constructor `Superstate::name()` that returns the state to enter
/// the superstate at. The keys, their defaults and the inputs are described in
/// the documentation of [`state_machine`](macro@state_machine).
///
/// The enclosing `#[state_machine]` attribute interprets the arguments and
/// removes this attribute from its output. Outside such a block this attribute
/// has no effect: it ignores its arguments and returns the item unchanged.
///
/// # Examples
///
/// ```ignore
/// #[superstate(initial = State::idle(), entry_action = enter_powered)]
/// fn powered(event: &Event) -> Outcome<State> {
///     match event {
///         Event::Reset(_) => Transition(State::idle()),
///         Event::Press(_) => Super,
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn superstate(
    _arguments: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    item
}

/// Mark an item as a mounted submachine inside a `#[state_machine]` impl block.
///
/// The function is the mount handler. Its name in PascalCase, or the `name`
/// key, is the name of the variants in both generated enums. The keys are
/// `machine` and `field`, which are required, and `factory`, `context`, `name`,
/// `superstate`, `entry_action`, `exit_action` and `exit_handler`. The field
/// holds the mounted machine inside the parent's storage, as an
/// `Option<Machine>` that is created and dropped with the mount, or as a plain
/// `Machine` that persists.
/// The keys, their defaults and the inputs are described in the documentation
/// of [`state_machine`](macro@state_machine).
///
/// The enclosing `#[state_machine]` attribute interprets the arguments and
/// removes this attribute from its output. Outside such a block this attribute
/// has no effect: it ignores its arguments and returns the item unchanged.
///
/// # Examples
///
/// ```ignore
/// #[submachine(
///     machine = meter::Meter,
///     field = backup,
///     context = sotu::event::unit_context
/// )]
/// fn backup(event: &Event) -> Outcome<State> {
///     match event {
///         Event::Close(_) => Transition(State::idle()),
///         Event::UseLive(_) | Event::UseBackup(_) | Event::Tick(_) => Super,
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn submachine(
    _arguments: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    item
}

/// Mark a function as an entry or exit action inside a `#[state_machine]` impl block.
///
/// An action is attached to a node with `entry_action = name` or
/// `exit_action = name` on `#[state]`, `#[superstate]` or `#[submachine]`. It
/// takes no arguments itself. Its inputs are an optional receiver, an optional
/// context, and references to the local storage of the node that names it. The
/// actions of a mount may also take the `submachine` input. An action may be
/// an `async fn`, which makes the machine awaitable.
///
/// The enclosing `#[state_machine]` attribute interprets the arguments and
/// removes this attribute from its output. Outside such a block this attribute
/// has no effect: it ignores its arguments and returns the item unchanged.
///
/// # Examples
///
/// ```ignore
/// #[state(entry_action = enter_on, local_storage(started: bool))]
/// fn on(event: &Event) -> Outcome<State> {
///     Handled
/// }
///
/// #[action]
/// fn enter_on(&mut self, started: &mut bool) {
///     *started = true;
/// }
/// ```
#[proc_macro_attribute]
pub fn action(
    _arguments: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    item
}

/// Mark a function as a submachine exit handler inside a `#[state_machine]` impl block.
///
/// A mount names the exit handler with `exit_handler = name`, and the handler
/// answers the exit values of the mounted machine with an outcome of the
/// parent, such as a transition or the parent's own `Exit`. It takes no
/// arguments itself. Its input of the exit value is passed by value and named
/// `exit` unless `exit_identifier` renames it, and it may also take the
/// receiver, the context and the `submachine` input. Several mounts may share
/// one exit handler, and an exit handler that no mount names is an error.
///
/// The enclosing `#[state_machine]` attribute interprets the arguments and
/// removes this attribute from its output. Outside such a block this attribute
/// has no effect: it ignores its arguments and returns the item unchanged.
///
/// # Examples
///
/// ```ignore
/// #[exit_handler]
/// fn door_exited(exit: door::DoorExit) -> Outcome<State> {
///     match exit {
///         door::DoorExit::Closed => Transition(State::washing()),
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn exit_handler(
    _arguments: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    item
}
