//! Parse the arguments of `#[state_machine]` and of the item attributes it consumes.

use syn::parse::{Parse, ParseStream};
use syn::{
    Attribute, Expr, ExprLit, Ident, Lit, Meta, Path, Token, Type, Visibility, parenthesized, token,
};

// MARK: Errors

/// Add `error` to the errors collected in `errors`.
pub(crate) fn push_error(errors: &mut Option<syn::Error>, error: syn::Error) {
    match errors {
        Some(existing) => existing.combine(error),
        None => *errors = Some(error),
    }
}

// MARK: Key-Value Arguments

/// Parse comma-separated `key = value` arguments whose keys come from `expected`.
///
/// Every unknown key, duplicate key and quoted value in the input is reported in
/// one combined error.
fn parse_key_values(input: ParseStream<'_>, expected: &[&str]) -> syn::Result<Vec<(Ident, Expr)>> {
    let mut entries: Vec<(Ident, Expr)> = Vec::new();
    let mut errors = None;

    // A syntax error leaves the stream in an unknown position, so it ends the
    // loop while keeping the errors found so far.
    while !input.is_empty() {
        let (key, value) = match parse_key_value(input, expected) {
            Ok(entry) => entry,
            Err(error) => {
                push_error(&mut errors, error);

                break;
            }
        };

        match validate_key_value(&key, &value, expected, &entries) {
            Ok(()) => entries.push((key, value)),
            Err(error) => push_error(&mut errors, error),
        }
    }

    match errors {
        Some(error) => Err(error),
        None => Ok(entries),
    }
}

/// Parse one `key = value` argument and the comma that separates it from the next.
///
/// A key outside `expected` that is not followed by `=` is reported as an
/// unknown argument at the key, since naming the missing `=` would point the
/// user at the wrong mistake.
fn parse_key_value(input: ParseStream<'_>, expected: &[&str]) -> syn::Result<(Ident, Expr)> {
    let key: Ident = input.parse()?;

    if !input.peek(Token![=]) && !expected.iter().any(|name| key == name) {
        return Err(syn::Error::new_spanned(
            &key,
            unknown_argument(&key, expected),
        ));
    }

    input.parse::<Token![=]>()?;

    let value: Expr = input.parse()?;

    if !input.is_empty() {
        input.parse::<Token![,]>()?;
    }

    Ok((key, value))
}

fn validate_key_value(
    key: &Ident,
    value: &Expr,
    expected: &[&str],
    entries: &[(Ident, Expr)],
) -> syn::Result<()> {
    if !expected.iter().any(|name| key == name) {
        return Err(syn::Error::new_spanned(
            key,
            unknown_argument(key, expected),
        ));
    }

    if entries.iter().any(|(existing, _)| existing == key) {
        return Err(syn::Error::new_spanned(
            key,
            format!("duplicate `{key}` item"),
        ));
    }

    if let Expr::Lit(ExprLit {
        lit: Lit::Str(literal),
        ..
    }) = value
    {
        return Err(syn::Error::new_spanned(
            literal,
            format!(
                "expected tokens, found string literal; write `{}` without quotes",
                literal.value()
            ),
        ));
    }

    Ok(())
}

fn unknown_argument(key: &Ident, expected: &[&str]) -> String {
    if expected.is_empty() {
        return format!("unknown argument `{key}`; this attribute takes no arguments");
    }

    let names: Vec<String> = expected.iter().map(|name| format!("`{name}`")).collect();

    format!(
        "unknown argument `{key}`; expected one of: {}",
        names.join(", ")
    )
}

// MARK: Machine Arguments

/// The arguments of `#[state_machine]`.
pub(crate) struct MachineArgs {
    /// The expression that builds the initial state.
    pub(crate) initial: Option<Expr>,
    /// The context type, which may use the lifetimes `'ctx` and `'data`.
    pub(crate) context: Option<Type>,
    /// The name of the event input, replacing `event`.
    pub(crate) event_identifier: Option<Ident>,
    /// The name of the context input, replacing `context`.
    pub(crate) context_identifier: Option<Ident>,
    /// The type of the exit points through which the machine can be left.
    pub(crate) exit: Option<Type>,
    /// The function called just before a node handles an event.
    pub(crate) before_dispatch: Option<Path>,
    /// The function called just after a node handled an event.
    pub(crate) after_dispatch: Option<Path>,
    /// The function called just before a transition runs its exit actions.
    pub(crate) before_transition: Option<Path>,
    /// The function called just after a transition ran its entry actions.
    pub(crate) after_transition: Option<Path>,
    /// The visibility of the generated types and constructors.
    pub(crate) visibility: Option<Visibility>,
    /// The options of the state enum.
    pub(crate) state: StateTypeArgs,
    /// The options of the superstate enum.
    pub(crate) superstate: SuperstateTypeArgs,
}

/// The arguments of `state(...)` in `#[state_machine]`.
#[derive(Default)]
pub(crate) struct StateTypeArgs {
    /// The name of the state enum, replacing `State`.
    pub(crate) name: Option<Ident>,
    /// The `custom` key, present when the user defines the state enum instead
    /// of the macro.
    pub(crate) custom: Option<Ident>,
    /// The traits to derive on the generated state enum.
    pub(crate) derives: Vec<Path>,
}

/// The arguments of `superstate(...)` in `#[state_machine]`.
#[derive(Default)]
pub(crate) struct SuperstateTypeArgs {
    /// The name of the superstate enum, replacing `Superstate`.
    pub(crate) name: Option<Ident>,
    /// The traits to derive on the generated superstate enum.
    pub(crate) derives: Vec<Path>,
}

impl StateTypeArgs {
    /// The keys of `state(...)`.
    const KEYS: &[&str] = &["name", "custom", "derive"];
}

impl SuperstateTypeArgs {
    /// The keys of `superstate(...)`.
    const KEYS: &[&str] = &["name", "derive"];
}

impl MachineArgs {
    /// The keys of `#[state_machine]`.
    const KEYS: &[&str] = &[
        "initial",
        "context",
        "event_identifier",
        "context_identifier",
        "exit",
        "before_dispatch",
        "after_dispatch",
        "before_transition",
        "after_transition",
        "visibility",
        "state",
        "superstate",
    ];

    /// The keys of `#[state_machine]` whose value is a type.
    const TYPE_KEYS: &[&str] = &["context", "exit"];

    /// Store the `key = Type` argument `key`, reporting a duplicate.
    fn set_type(&mut self, key: &Ident, ty: Type) -> syn::Result<()> {
        let slot = if key == "context" {
            &mut self.context
        } else {
            &mut self.exit
        };

        if slot.is_some() {
            return Err(syn::Error::new_spanned(
                key,
                format!("duplicate `{key}` item"),
            ));
        }

        *slot = Some(ty);

        Ok(())
    }

    /// Return the field that stores the hook `key`, or `None` if `key` names no hook.
    fn hook_slot(&mut self, key: &Ident) -> Option<&mut Option<Path>> {
        if key == "before_dispatch" {
            Some(&mut self.before_dispatch)
        } else if key == "after_dispatch" {
            Some(&mut self.after_dispatch)
        } else if key == "before_transition" {
            Some(&mut self.before_transition)
        } else if key == "after_transition" {
            Some(&mut self.after_transition)
        } else {
            None
        }
    }

    /// Store the `visibility = vis` argument `key`, reporting a duplicate.
    fn set_visibility(&mut self, key: &Ident, visibility: Visibility) -> syn::Result<()> {
        if self.visibility.is_some() {
            return Err(syn::Error::new_spanned(
                key,
                format!("duplicate `{key}` item"),
            ));
        }

        self.visibility = Some(visibility);

        Ok(())
    }

    /// Store the options of the `state(...)` or `superstate(...)` argument `key`.
    ///
    /// The superstate keys exclude `custom`, so its `custom` is always `None`.
    fn set_type_options(&mut self, key: &Ident, options: StateTypeArgs) {
        if key == "state" {
            self.state = options;
        } else {
            self.superstate = SuperstateTypeArgs {
                name: options.name,
                derives: options.derives,
            };
        }
    }

    /// Store the `key = value` argument `key`.
    fn set_key_value(&mut self, key: &Ident, value: &Expr) -> syn::Result<()> {
        if key == "initial" {
            self.initial = Some(value.clone());

            return Ok(());
        }

        if key == "state" || key == "superstate" {
            return Err(syn::Error::new_spanned(
                key,
                format!("`{key}` takes a list, as in `{key}(name = Name)`"),
            ));
        }

        if let Some(slot) = self.hook_slot(key) {
            *slot = Some(path_value(value)?);

            return Ok(());
        }

        let ident = ident_value(value)?;

        if key == "event_identifier" {
            self.event_identifier = Some(ident);
        } else {
            self.context_identifier = Some(ident);
        }

        Ok(())
    }
}

impl Parse for MachineArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut arguments = Self {
            initial: None,
            context: None,
            event_identifier: None,
            context_identifier: None,
            exit: None,
            before_dispatch: None,
            after_dispatch: None,
            before_transition: None,
            after_transition: None,
            visibility: None,
            state: StateTypeArgs::default(),
            superstate: SuperstateTypeArgs::default(),
        };

        let mut entries: Vec<(Ident, Expr)> = Vec::new();
        let mut lists: Vec<Ident> = Vec::new();
        let mut errors = None;

        // A syntax error leaves the stream in an unknown position, so it ends the
        // loop while keeping the errors found so far. The context and exit are
        // types and the visibility is a visibility, none of which parse as the
        // expression every other value is.
        while !input.is_empty() {
            if is_type_argument(input, "visibility") {
                match parse_visibility_argument(input) {
                    Ok((key, visibility)) => {
                        if let Err(error) = arguments.set_visibility(&key, visibility) {
                            push_error(&mut errors, error);
                        }
                    }
                    Err(error) => {
                        push_error(&mut errors, error);

                        break;
                    }
                }

                continue;
            }

            if is_list_argument(input, "state") || is_list_argument(input, "superstate") {
                match parse_type_options_argument(input, &mut errors) {
                    Ok((key, _)) if lists.contains(&key) => push_error(
                        &mut errors,
                        syn::Error::new_spanned(&key, format!("duplicate `{key}` item")),
                    ),
                    Ok((key, options)) => {
                        arguments.set_type_options(&key, options);
                        lists.push(key);
                    }
                    Err(error) => {
                        push_error(&mut errors, error);

                        break;
                    }
                }

                continue;
            }

            if Self::TYPE_KEYS
                .iter()
                .any(|key| is_type_argument(input, key))
            {
                match parse_type_argument(input) {
                    Ok((key, ty)) => {
                        if let Err(error) = arguments.set_type(&key, ty) {
                            push_error(&mut errors, error);
                        }
                    }
                    Err(error) => {
                        push_error(&mut errors, error);

                        break;
                    }
                }

                continue;
            }

            let (key, value) = match parse_key_value(input, Self::KEYS) {
                Ok(entry) => entry,
                Err(error) => {
                    push_error(&mut errors, error);

                    break;
                }
            };

            let validated = validate_key_value(&key, &value, Self::KEYS, &entries)
                .and_then(|()| arguments.set_key_value(&key, &value));

            match validated {
                Ok(()) => entries.push((key, value)),
                Err(error) => push_error(&mut errors, error),
            }
        }

        match errors {
            Some(error) => Err(error),
            None => Ok(arguments),
        }
    }
}

/// Return whether the next argument in `input` is written `key = Type`.
fn is_type_argument(input: ParseStream<'_>, key: &str) -> bool {
    let fork = input.fork();

    fork.parse::<Ident>().is_ok_and(|ident| ident == key) && fork.peek(Token![=])
}

/// Parse one `visibility = vis` argument and the comma that separates it from the next.
fn parse_visibility_argument(input: ParseStream<'_>) -> syn::Result<(Ident, Visibility)> {
    let key: Ident = input.parse()?;

    input.parse::<Token![=]>()?;

    let visibility: Visibility = input.parse()?;

    if !input.is_empty() {
        input.parse::<Token![,]>()?;
    }

    Ok((key, visibility))
}

/// Parse a `state(...)` or `superstate(...)` argument and the comma that follows it.
///
/// The errors inside the parentheses go to `errors`, because they leave the
/// stream after the parentheses intact. Only a syntax error outside them is
/// returned.
fn parse_type_options_argument(
    input: ParseStream<'_>,
    errors: &mut Option<syn::Error>,
) -> syn::Result<(Ident, StateTypeArgs)> {
    let key: Ident = input.parse()?;
    let content;

    parenthesized!(content in input);

    let keys = if key == "state" {
        StateTypeArgs::KEYS
    } else {
        SuperstateTypeArgs::KEYS
    };

    let mut options = StateTypeArgs::default();
    let mut seen: Vec<Ident> = Vec::new();

    // A syntax error leaves the stream in an unknown position, so it ends the
    // loop while keeping the errors found so far.
    while !content.is_empty() {
        if let Err(error) = parse_type_option(&content, keys, &mut options, &mut seen, errors) {
            push_error(errors, error);

            break;
        }
    }

    if !input.is_empty() {
        input.parse::<Token![,]>()?;
    }

    Ok((key, options))
}

/// Parse one option of a `state(...)` or `superstate(...)` argument into `options`.
///
/// An unknown or duplicate key goes to `errors`, and the tokens of an unknown
/// key are skipped up to the next comma. Only a syntax error is returned.
fn parse_type_option(
    input: ParseStream<'_>,
    keys: &[&str],
    options: &mut StateTypeArgs,
    seen: &mut Vec<Ident>,
    errors: &mut Option<syn::Error>,
) -> syn::Result<()> {
    let key: Ident = input.parse()?;

    if !keys.iter().any(|name| key == name) {
        push_error(
            errors,
            syn::Error::new_spanned(&key, unknown_argument(&key, keys)),
        );

        while !input.is_empty() && !input.peek(Token![,]) {
            input.parse::<proc_macro2::TokenTree>()?;
        }
    } else {
        if seen.contains(&key) {
            push_error(
                errors,
                syn::Error::new_spanned(&key, format!("duplicate `{key}` item")),
            );
        }

        if key == "name" {
            input.parse::<Token![=]>()?;
            options.name = Some(input.parse()?);
        } else if key == "custom" {
            options.custom = Some(key.clone());
        } else {
            let content;

            parenthesized!(content in input);

            let derives = content.parse_terminated(Path::parse_mod_style, Token![,])?;

            options.derives.extend(derives);
        }

        seen.push(key);
    }

    if !input.is_empty() {
        input.parse::<Token![,]>()?;
    }

    Ok(())
}

/// Return whether the next argument in `input` is written `key(...)`.
fn is_list_argument(input: ParseStream<'_>, key: &str) -> bool {
    let fork = input.fork();

    fork.parse::<Ident>().is_ok_and(|ident| ident == key) && fork.peek(token::Paren)
}

/// Parse one `key = Type` argument and the comma that separates it from the next.
fn parse_type_argument(input: ParseStream<'_>) -> syn::Result<(Ident, Type)> {
    let key: Ident = input.parse()?;

    input.parse::<Token![=]>()?;

    let ty: Type = input.parse()?;

    if !input.is_empty() {
        input.parse::<Token![,]>()?;
    }

    Ok((key, ty))
}

// MARK: Item Attributes

/// An attribute of `sotu` on an item inside a `#[state_machine]` impl block.
pub(crate) enum ItemAttribute {
    /// The item is a state handler.
    State(StateArgs),
    /// The item is a superstate handler.
    Superstate(SuperstateArgs),
    /// The item is the handler of a mounted submachine.
    Submachine(SubmachineArgs),
    /// The item is an entry or exit action.
    Action,
}

/// The arguments of `#[state]`.
#[derive(Default)]
pub(crate) struct StateArgs {
    /// The name of the generated variant, replacing the one derived from the handler.
    pub(crate) name: Option<Ident>,
    /// The handler name of the enclosing superstate.
    pub(crate) superstate: Option<Ident>,
    /// The action to run when the state is entered.
    pub(crate) entry_action: Option<Ident>,
    /// The action to run when the state is exited.
    pub(crate) exit_action: Option<Ident>,
    /// The extra local storage fields, in declaration order.
    pub(crate) local_storage: Vec<(Ident, Type)>,
}

/// The arguments of `#[superstate]`.
#[derive(Default)]
pub(crate) struct SuperstateArgs {
    /// The name of the generated variant, replacing the one derived from the handler.
    pub(crate) name: Option<Ident>,
    /// The handler name of the enclosing superstate.
    pub(crate) superstate: Option<Ident>,
    /// The action to run when the superstate is entered.
    pub(crate) entry_action: Option<Ident>,
    /// The action to run when the superstate is exited.
    pub(crate) exit_action: Option<Ident>,
    /// The state expression that the generated `initial` constructor returns.
    pub(crate) initial: Option<Expr>,
    /// The extra borrowed fields, in declaration order.
    pub(crate) local_storage: Vec<(Ident, Type)>,
}

/// The arguments of `#[submachine]`.
#[derive(Default)]
pub(crate) struct SubmachineArgs {
    /// The type of the mounted machine.
    pub(crate) machine: Option<Type>,
    /// The field of the shared storage that holds the mounted machine.
    pub(crate) field: Option<Ident>,
    /// The name of the generated variants, replacing the one derived from the handler.
    pub(crate) name: Option<Ident>,
    /// The handler name of the enclosing superstate.
    pub(crate) superstate: Option<Ident>,
}

impl StateArgs {
    /// The keys of `#[state]`.
    const KEYS: &[&str] = &[
        "name",
        "superstate",
        "entry_action",
        "exit_action",
        "local_storage",
    ];

    /// Store the `key = value` argument `key`.
    fn set_key_value(&mut self, key: &Ident, value: &Expr) -> syn::Result<()> {
        let ident = ident_value(value)?;

        if key == "name" {
            self.name = Some(ident);
        } else if key == "superstate" {
            self.superstate = Some(ident);
        } else if key == "entry_action" {
            self.entry_action = Some(ident);
        } else {
            self.exit_action = Some(ident);
        }

        Ok(())
    }
}

impl Parse for StateArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut arguments = Self::default();

        let local_storage = parse_node_arguments(input, Self::KEYS, |key, value| {
            arguments.set_key_value(key, value)
        })?;

        arguments.local_storage = local_storage;

        Ok(arguments)
    }
}

impl SuperstateArgs {
    /// The keys of `#[superstate]`.
    const KEYS: &[&str] = &[
        "name",
        "superstate",
        "entry_action",
        "exit_action",
        "initial",
        "local_storage",
    ];

    /// Store the `key = value` argument `key`.
    fn set_key_value(&mut self, key: &Ident, value: &Expr) -> syn::Result<()> {
        if key == "initial" {
            self.initial = Some(value.clone());

            return Ok(());
        }

        let ident = ident_value(value)?;

        if key == "name" {
            self.name = Some(ident);
        } else if key == "superstate" {
            self.superstate = Some(ident);
        } else if key == "entry_action" {
            self.entry_action = Some(ident);
        } else {
            self.exit_action = Some(ident);
        }

        Ok(())
    }
}

impl Parse for SuperstateArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut arguments = Self::default();

        let local_storage = parse_node_arguments(input, Self::KEYS, |key, value| {
            arguments.set_key_value(key, value)
        })?;

        arguments.local_storage = local_storage;

        Ok(arguments)
    }
}

impl SubmachineArgs {
    /// The keys of `#[submachine]`.
    const KEYS: &[&str] = &["machine", "field", "name", "superstate"];

    /// Store the `key = ident` argument `key`.
    fn set_key_value(&mut self, key: &Ident, value: &Expr) -> syn::Result<()> {
        let ident = ident_value(value)?;

        if key == "field" {
            self.field = Some(ident);
        } else if key == "name" {
            self.name = Some(ident);
        } else {
            self.superstate = Some(ident);
        }

        Ok(())
    }
}

impl Parse for SubmachineArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut arguments = Self::default();
        let mut entries: Vec<(Ident, Expr)> = Vec::new();
        let mut errors = None;

        // A syntax error leaves the stream in an unknown position, so it ends the
        // loop while keeping the errors found so far. The machine is a type, which
        // does not parse as the expression every other value is.
        while !input.is_empty() {
            if is_type_argument(input, "machine") {
                match parse_type_argument(input) {
                    Ok((key, _)) if arguments.machine.is_some() => push_error(
                        &mut errors,
                        syn::Error::new_spanned(&key, format!("duplicate `{key}` item")),
                    ),
                    Ok((_, machine)) => arguments.machine = Some(machine),
                    Err(error) => {
                        push_error(&mut errors, error);

                        break;
                    }
                }

                continue;
            }

            let (key, value) = match parse_key_value(input, Self::KEYS) {
                Ok(entry) => entry,
                Err(error) => {
                    push_error(&mut errors, error);

                    break;
                }
            };

            let validated = validate_key_value(&key, &value, Self::KEYS, &entries)
                .and_then(|()| arguments.set_key_value(&key, &value));

            match validated {
                Ok(()) => entries.push((key, value)),
                Err(error) => push_error(&mut errors, error),
            }
        }

        match errors {
            Some(error) => Err(error),
            None => Ok(arguments),
        }
    }
}

/// Parse the arguments of a node attribute, handing each `key = value` to `set`.
///
/// Returns the fields of the `local_storage(...)` argument. Every unknown key,
/// duplicate key and quoted value is reported in one combined error.
fn parse_node_arguments(
    input: ParseStream<'_>,
    keys: &[&str],
    mut set: impl FnMut(&Ident, &Expr) -> syn::Result<()>,
) -> syn::Result<Vec<(Ident, Type)>> {
    let mut local_storage = None;
    let mut entries: Vec<(Ident, Expr)> = Vec::new();
    let mut errors = None;

    // A syntax error leaves the stream in an unknown position, so it ends the
    // loop while keeping the errors found so far.
    while !input.is_empty() {
        if is_local_storage_list(input) {
            match parse_local_storage(input) {
                Ok((key, _)) if local_storage.is_some() => push_error(
                    &mut errors,
                    syn::Error::new_spanned(&key, format!("duplicate `{key}` item")),
                ),
                Ok((_, fields)) => local_storage = Some(fields),
                Err(error) => {
                    push_error(&mut errors, error);

                    break;
                }
            }

            continue;
        }

        let (key, value) = match parse_key_value(input, keys) {
            Ok(entry) => entry,
            Err(error) => {
                push_error(&mut errors, error);

                break;
            }
        };

        let validated = validate_key_value(&key, &value, keys, &entries).and_then(|()| {
            if key == "local_storage" {
                return Err(syn::Error::new_spanned(
                    &key,
                    "`local_storage` takes a list, as in `local_storage(field: Type)`",
                ));
            }

            set(&key, &value)
        });

        match validated {
            Ok(()) => entries.push((key, value)),
            Err(error) => push_error(&mut errors, error),
        }
    }

    match errors {
        Some(error) => Err(error),
        None => Ok(local_storage.unwrap_or_default()),
    }
}

/// Return the identifier written as `value`.
fn ident_value(value: &Expr) -> syn::Result<Ident> {
    if let Expr::Path(path) = value
        && let Some(ident) = path.path.get_ident()
    {
        return Ok(ident.clone());
    }

    Err(syn::Error::new_spanned(value, "expected an identifier"))
}

/// Return the path written as `value`, as in `Self::record`.
fn path_value(value: &Expr) -> syn::Result<Path> {
    if let Expr::Path(path) = value
        && path.qself.is_none()
    {
        return Ok(path.path.clone());
    }

    Err(syn::Error::new_spanned(
        value,
        "expected a path to a function",
    ))
}

/// Return whether the next argument in `input` is written `local_storage(...)`.
fn is_local_storage_list(input: ParseStream<'_>) -> bool {
    let fork = input.fork();

    fork.parse::<Ident>()
        .is_ok_and(|key| key == "local_storage")
        && fork.peek(token::Paren)
}

/// Parse a `local_storage(field: Type, ...)` argument and the comma that follows it.
fn parse_local_storage(input: ParseStream<'_>) -> syn::Result<(Ident, Vec<(Ident, Type)>)> {
    let key: Ident = input.parse()?;
    let content;

    parenthesized!(content in input);

    let fields = content.parse_terminated(parse_local_storage_field, Token![,])?;

    if !input.is_empty() {
        input.parse::<Token![,]>()?;
    }

    Ok((key, fields.into_iter().collect()))
}

fn parse_local_storage_field(input: ParseStream<'_>) -> syn::Result<(Ident, Type)> {
    let name: Ident = input.parse()?;

    input.parse::<Token![:]>()?;

    let ty: Type = input.parse()?;

    Ok((name, ty))
}

/// Parse `attribute` if it is an attribute of `sotu`.
///
/// Returns `Ok(None)` for any attribute whose last path segment is not one of
/// `state`, `superstate`, `submachine`, `action` or `exit_handler`.
pub(crate) fn parse_item_attribute(attribute: &Attribute) -> syn::Result<Option<ItemAttribute>> {
    let Some(segment) = attribute.path().segments.last() else {
        return Ok(None);
    };

    let name = segment.ident.to_string();

    match name.as_str() {
        "state" => {
            parse_state_args(attribute).map(|arguments| Some(ItemAttribute::State(arguments)))
        }
        "superstate" => parse_superstate_args(attribute)
            .map(|arguments| Some(ItemAttribute::Superstate(arguments))),
        "action" => parse_action_args(attribute).map(|()| Some(ItemAttribute::Action)),
        "submachine" => parse_submachine_args(attribute)
            .map(|arguments| Some(ItemAttribute::Submachine(arguments))),
        "exit_handler" => Err(syn::Error::new_spanned(
            attribute,
            format!("`#[{name}]` is not supported here"),
        )),
        _ => Ok(None),
    }
}

fn parse_state_args(attribute: &Attribute) -> syn::Result<StateArgs> {
    match &attribute.meta {
        Meta::Path(_) => Ok(StateArgs::default()),
        Meta::List(list) => list.parse_args(),
        Meta::NameValue(name_value) => Err(syn::Error::new_spanned(
            name_value,
            "`#[state]` takes no value",
        )),
    }
}

fn parse_superstate_args(attribute: &Attribute) -> syn::Result<SuperstateArgs> {
    match &attribute.meta {
        Meta::Path(_) => Ok(SuperstateArgs::default()),
        Meta::List(list) => list.parse_args(),
        Meta::NameValue(name_value) => Err(syn::Error::new_spanned(
            name_value,
            "`#[superstate]` takes no value",
        )),
    }
}

fn parse_submachine_args(attribute: &Attribute) -> syn::Result<SubmachineArgs> {
    match &attribute.meta {
        Meta::Path(_) => Ok(SubmachineArgs::default()),
        Meta::List(list) => list.parse_args(),
        Meta::NameValue(name_value) => Err(syn::Error::new_spanned(
            name_value,
            "`#[submachine]` takes no value",
        )),
    }
}

fn parse_action_args(attribute: &Attribute) -> syn::Result<()> {
    match &attribute.meta {
        Meta::Path(_) => Ok(()),
        Meta::List(list) => list
            .parse_args_with(|input: ParseStream<'_>| parse_key_values(input, &[]))
            .map(|_| ()),
        Meta::NameValue(name_value) => Err(syn::Error::new_spanned(
            name_value,
            "`#[action]` takes no value",
        )),
    }
}

// MARK: Tests

#[cfg(test)]
mod tests {
    use quote::quote;
    use syn::parse_quote;

    use super::*;

    fn argument_error(source: &str) -> String {
        match syn::parse_str::<MachineArgs>(source) {
            Ok(_) => panic!("`{source}` should be rejected"),
            Err(error) => error.to_string(),
        }
    }

    #[test]
    fn machine_arguments_read_initial() {
        let arguments: MachineArgs = syn::parse_str("initial = State::idle()").unwrap();
        let initial = arguments.initial.unwrap();

        assert_eq!(quote!(#initial).to_string(), "State :: idle ()");
    }

    #[test]
    fn machine_arguments_read_context_and_identifiers() {
        let arguments: MachineArgs = syn::parse_str(
            "initial = State::a(), context = Sink<'ctx, 'data>, event_identifier = input, \
             context_identifier = sink",
        )
        .unwrap();

        let initial = arguments.initial.unwrap();
        let context = arguments.context.unwrap();

        assert_eq!(quote!(#initial).to_string(), "State :: a ()");
        assert_eq!(quote!(#context).to_string(), "Sink < 'ctx , 'data >");
        assert!(
            arguments
                .event_identifier
                .is_some_and(|name| name == "input")
        );
        assert!(
            arguments
                .context_identifier
                .is_some_and(|name| name == "sink")
        );
    }

    #[test]
    fn machine_arguments_read_exit_and_hooks() {
        let arguments: MachineArgs = syn::parse_str(
            "initial = State::a(), exit = Done, before_dispatch = Self::before, \
             after_dispatch = Self::after, before_transition = Self::leaving, \
             after_transition = Self::arrived",
        )
        .unwrap();

        let exit = arguments.exit.unwrap();
        let before_dispatch = arguments.before_dispatch.unwrap();
        let after_dispatch = arguments.after_dispatch.unwrap();
        let before_transition = arguments.before_transition.unwrap();
        let after_transition = arguments.after_transition.unwrap();

        assert_eq!(quote!(#exit).to_string(), "Done");
        assert_eq!(quote!(#before_dispatch).to_string(), "Self :: before");
        assert_eq!(quote!(#after_dispatch).to_string(), "Self :: after");
        assert_eq!(quote!(#before_transition).to_string(), "Self :: leaving");
        assert_eq!(quote!(#after_transition).to_string(), "Self :: arrived");
    }

    #[test]
    fn machine_arguments_read_type_options() {
        let arguments: MachineArgs = syn::parse_str(
            "initial = MyState::a(), visibility = pub(crate), \
             state(name = MyState, derive(Debug, PartialEq)), \
             superstate(name = MySuperstate, derive(Debug))",
        )
        .unwrap();

        let visibility = arguments.visibility.unwrap();
        let paths = |derives: &[Path]| -> Vec<String> {
            derives
                .iter()
                .map(|path| quote!(#path).to_string())
                .collect()
        };

        assert_eq!(quote!(#visibility).to_string(), "pub (crate)");
        assert!(arguments.state.name.is_some_and(|name| name == "MyState"));
        assert!(arguments.state.custom.is_none());
        assert_eq!(paths(&arguments.state.derives), ["Debug", "PartialEq"]);
        assert!(
            arguments
                .superstate
                .name
                .is_some_and(|name| name == "MySuperstate")
        );
        assert_eq!(paths(&arguments.superstate.derives), ["Debug"]);

        let custom: MachineArgs =
            syn::parse_str("initial = Mine::Off {}, state(custom, name = Mine)").unwrap();

        assert!(custom.state.custom.is_some_and(|key| key == "custom"));
        assert!(custom.state.name.is_some_and(|name| name == "Mine"));
        assert!(custom.state.derives.is_empty());
    }

    #[test]
    fn unknown_argument_is_rejected() {
        let message = argument_error("initial = State::a(), bogus = 1");

        assert!(
            message.starts_with("unknown argument `bogus`"),
            "unexpected message: {message}"
        );
    }

    #[test]
    fn duplicate_argument_is_rejected() {
        let message = argument_error("initial = State::a(), initial = State::b()");

        assert_eq!(message, "duplicate `initial` item");
    }

    #[test]
    fn string_literal_argument_is_rejected() {
        let message = argument_error(r#"initial = "State::a()""#);

        assert!(
            message.starts_with("expected tokens, found string literal"),
            "unexpected message: {message}"
        );
        assert!(
            message.contains("`State::a()`"),
            "unexpected message: {message}"
        );
    }

    #[test]
    fn state_attribute_accepts_no_arguments() {
        let bare: Attribute = parse_quote!(#[state]);
        let empty: Attribute = parse_quote!(#[state()]);

        assert!(matches!(
            parse_item_attribute(&bare),
            Ok(Some(ItemAttribute::State(_)))
        ));
        assert!(matches!(
            parse_item_attribute(&empty),
            Ok(Some(ItemAttribute::State(_)))
        ));
    }

    #[test]
    fn state_arguments_read_every_key() {
        let attribute: Attribute = parse_quote! {
            #[state(
                name = Lit,
                entry_action = enter,
                exit_action = leave,
                local_storage(started: bool, count: u8)
            )]
        };

        let Ok(Some(ItemAttribute::State(arguments))) = parse_item_attribute(&attribute) else {
            panic!("the attribute should parse as a state");
        };

        let fields: Vec<String> = arguments
            .local_storage
            .iter()
            .map(|(name, ty)| quote!(#name: #ty).to_string())
            .collect();

        assert!(arguments.name.is_some_and(|name| name == "Lit"));
        assert!(arguments.entry_action.is_some_and(|name| name == "enter"));
        assert!(arguments.exit_action.is_some_and(|name| name == "leave"));
        assert_eq!(fields, ["started : bool", "count : u8"]);
    }

    #[test]
    fn superstate_arguments_read_every_key() {
        let attribute: Attribute = parse_quote! {
            #[superstate(
                name = Lit,
                superstate = outer,
                entry_action = enter,
                exit_action = leave,
                initial = State::on(0),
                local_storage(started: &'sub mut bool, count: &'sub u8)
            )]
        };

        let Ok(Some(ItemAttribute::Superstate(arguments))) = parse_item_attribute(&attribute)
        else {
            panic!("the attribute should parse as a superstate");
        };

        let initial = arguments.initial;

        let fields: Vec<String> = arguments
            .local_storage
            .iter()
            .map(|(name, ty)| quote!(#name: #ty).to_string())
            .collect();

        assert!(arguments.name.is_some_and(|name| name == "Lit"));
        assert!(arguments.superstate.is_some_and(|name| name == "outer"));
        assert!(arguments.entry_action.is_some_and(|name| name == "enter"));
        assert!(arguments.exit_action.is_some_and(|name| name == "leave"));
        assert_eq!(quote!(#initial).to_string(), "State :: on (0)");
        assert_eq!(fields, ["started : & 'sub mut bool", "count : & 'sub u8"]);
    }

    #[test]
    fn submachine_arguments_read_every_key() {
        let attribute: Attribute = parse_quote! {
            #[submachine(machine = door::Door, field = door, name = Loading, superstate = group)]
        };

        let Ok(Some(ItemAttribute::Submachine(arguments))) = parse_item_attribute(&attribute)
        else {
            panic!("the attribute should parse as a submachine");
        };

        let machine = arguments.machine;

        assert_eq!(quote!(#machine).to_string(), "door :: Door");
        assert!(arguments.field.is_some_and(|name| name == "door"));
        assert!(arguments.name.is_some_and(|name| name == "Loading"));
        assert!(arguments.superstate.is_some_and(|name| name == "group"));
    }

    #[test]
    fn action_attribute_accepts_no_arguments() {
        let attribute: Attribute = parse_quote!(#[action]);

        assert!(matches!(
            parse_item_attribute(&attribute),
            Ok(Some(ItemAttribute::Action))
        ));
    }

    #[test]
    fn foreign_attribute_is_ignored() {
        let attribute: Attribute = parse_quote!(#[inline]);

        assert!(matches!(parse_item_attribute(&attribute), Ok(None)));
    }
}
