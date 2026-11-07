//! Parse the arguments of `#[state_machine]` and of the item attributes it consumes.

use syn::parse::{Parse, ParseStream};
use syn::{Attribute, Expr, ExprLit, Ident, Lit, Meta, Token, Type, parenthesized, token};

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
        let (key, value) = match parse_key_value(input) {
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
fn parse_key_value(input: ParseStream<'_>) -> syn::Result<(Ident, Expr)> {
    let key: Ident = input.parse()?;

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
}

impl Parse for MachineArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut initial = None;

        for (key, value) in parse_key_values(input, &["initial"])? {
            if key == "initial" {
                initial = Some(value);
            }
        }

        Ok(Self { initial })
    }
}

// MARK: Item Attributes

/// An attribute of `sotu` on an item inside a `#[state_machine]` impl block.
pub(crate) enum ItemAttribute {
    /// The item is a state handler.
    State(StateArgs),
    /// The item is a superstate handler.
    Superstate(SuperstateArgs),
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

        let (key, value) = match parse_key_value(input) {
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
        "submachine" | "exit_handler" => Err(syn::Error::new_spanned(
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
