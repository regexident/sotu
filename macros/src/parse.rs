//! Parse the arguments of `#[state_machine]` and of the item attributes it consumes.

use syn::parse::{Parse, ParseStream};
use syn::{Attribute, Expr, ExprLit, Ident, Lit, Meta, Token};

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
}

/// The arguments of `#[state]`.
pub(crate) struct StateArgs {}

impl Parse for StateArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        parse_key_values(input, &[])?;

        Ok(Self {})
    }
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
        "superstate" | "submachine" | "action" | "exit_handler" => Err(syn::Error::new_spanned(
            attribute,
            format!("`#[{name}]` is not supported here"),
        )),
        _ => Ok(None),
    }
}

fn parse_state_args(attribute: &Attribute) -> syn::Result<StateArgs> {
    match &attribute.meta {
        Meta::Path(_) => Ok(StateArgs {}),
        Meta::List(list) => list.parse_args(),
        Meta::NameValue(name_value) => Err(syn::Error::new_spanned(
            name_value,
            "`#[state]` takes no value",
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
    fn foreign_attribute_is_ignored() {
        let attribute: Attribute = parse_quote!(#[inline]);

        assert!(matches!(parse_item_attribute(&attribute), Ok(None)));
    }
}
