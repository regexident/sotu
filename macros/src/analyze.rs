//! Validate and classify a `#[state_machine]` impl block.

use proc_macro2::Span;
use syn::ext::IdentExt;
use syn::{
    Attribute, Expr, FnArg, Ident, ImplItem, ImplItemFn, ItemImpl, Pat, PatType, Type, parse_quote,
};

use crate::parse::{ItemAttribute, MachineArgs, parse_item_attribute, push_error};

// MARK: Model

/// A validated `#[state_machine]` impl block.
pub(crate) struct Model {
    /// The type the impl block belongs to.
    pub(crate) shared_storage: Type,
    /// The expression that builds the initial state.
    pub(crate) initial: Expr,
    /// The event type of the machine.
    pub(crate) event_type: Type,
    /// The states in declaration order.
    pub(crate) states: Vec<StateModel>,
    /// The impl block with the attributes of `sotu` removed.
    pub(crate) item: ItemImpl,
}

/// A state handler of the machine.
pub(crate) struct StateModel {
    /// The name of the handler function.
    pub(crate) handler: Ident,
    /// The name of the generated enum variant.
    pub(crate) variant: Ident,
    /// The name of the generated constructor.
    pub(crate) constructor: Ident,
    /// The classified inputs of the handler, in declaration order.
    pub(crate) inputs: Vec<InputKind>,
    /// The doc comments of the handler.
    pub(crate) docs: Vec<Attribute>,
}

/// What an input of a state handler receives.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum InputKind {
    /// The receiver, which is the shared storage.
    Receiver,
    /// The input named `event`.
    Event,
}

// MARK: Analysis

/// Validate `item` and classify its handlers into a [`Model`].
///
/// A missing `initial` and the errors found in `item` are combined into one
/// result. The inputs of a handler are not checked when its `#[state(...)]`
/// attribute is invalid, because the item is then not classified as a state.
pub(crate) fn analyze(arguments: MachineArgs, mut item: ItemImpl) -> syn::Result<Model> {
    let mut errors = None;
    let mut states = Vec::new();
    let mut event_type = None;

    for impl_item in &mut item.items {
        if !take_item_attributes(impl_item, &mut errors) {
            continue;
        }

        if let ImplItem::Fn(function) = impl_item {
            states.push(analyze_state(function, &mut event_type, &mut errors));
        } else {
            push_error(
                &mut errors,
                syn::Error::new_spanned(&*impl_item, "`#[state]` can only be applied to functions"),
            );
        }
    }

    let Some(initial) = arguments.initial else {
        let mut error = syn::Error::new(
            Span::call_site(),
            "no initial state defined; add `initial = State::name()` to `#[state_machine]`",
        );

        if let Some(other_errors) = errors {
            error.combine(other_errors);
        }

        return Err(error);
    };

    if let Some(error) = errors {
        return Err(error);
    }

    Ok(Model {
        shared_storage: (*item.self_ty).clone(),
        initial,
        event_type: event_type.unwrap_or_else(|| parse_quote!(())),
        states,
        item,
    })
}

/// Remove the attributes of `sotu` from `item`, returning whether one was `#[state]`.
fn take_item_attributes(item: &mut ImplItem, errors: &mut Option<syn::Error>) -> bool {
    let attributes = match item {
        ImplItem::Const(item) => &mut item.attrs,
        ImplItem::Fn(item) => &mut item.attrs,
        ImplItem::Type(item) => &mut item.attrs,
        ImplItem::Macro(item) => &mut item.attrs,
        _ => return false,
    };

    let mut is_state = false;

    attributes.retain(|attribute| match parse_item_attribute(attribute) {
        Ok(None) => true,
        Ok(Some(ItemAttribute::State(_))) => {
            is_state = true;

            false
        }
        Err(error) => {
            push_error(errors, error);

            false
        }
    });

    is_state
}

fn analyze_state(
    function: &ImplItemFn,
    event_type: &mut Option<Type>,
    errors: &mut Option<syn::Error>,
) -> StateModel {
    let handler = function.sig.ident.clone();
    let variant = pascal_case(&handler);
    let constructor = snake_case(&variant);
    let mut inputs = Vec::new();

    for input in &function.sig.inputs {
        match input {
            FnArg::Receiver(_) => inputs.push(InputKind::Receiver),
            FnArg::Typed(typed) => match event_referent(typed) {
                Ok(referent) => {
                    inputs.push(InputKind::Event);
                    event_type.get_or_insert_with(|| referent.clone());
                }
                Err(error) => push_error(errors, error),
            },
        }
    }

    let docs = function
        .attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident("doc"))
        .cloned()
        .collect();

    StateModel {
        handler,
        variant,
        constructor,
        inputs,
        docs,
    }
}

/// Return the type behind the reference of the `event` input `typed`.
///
/// Fails if `typed` is any other input, or if `event` is not a shared reference.
fn event_referent(typed: &PatType) -> syn::Result<&Type> {
    let is_event = matches!(&*typed.pat, Pat::Ident(pattern) if pattern.ident == "event");

    if !is_event {
        return Err(syn::Error::new_spanned(
            typed,
            "unsupported input; state handlers take a receiver and `event`",
        ));
    }

    match &*typed.ty {
        Type::Reference(reference) if reference.mutability.is_none() => Ok(&reference.elem),
        ty => Err(syn::Error::new_spanned(
            ty,
            "the `event` input must be a shared reference, as in `event: &Event`",
        )),
    }
}

// MARK: Names

/// Return the PascalCase form of a snake_case `ident`, as in `door_open` to `DoorOpen`.
pub(crate) fn pascal_case(ident: &Ident) -> Ident {
    let mut name = String::new();

    for word in ident.unraw().to_string().split('_') {
        let mut characters = word.chars();

        if let Some(first) = characters.next() {
            name.extend(first.to_uppercase());
            name.push_str(characters.as_str());
        }
    }

    Ident::new(&name, ident.span())
}

/// Return the snake_case form of a PascalCase `ident`, as in `DoorOpen` to `door_open`.
pub(crate) fn snake_case(ident: &Ident) -> Ident {
    let mut name = String::new();

    for character in ident.unraw().to_string().chars() {
        if character.is_uppercase() && !name.is_empty() {
            name.push('_');
        }

        name.extend(character.to_lowercase());
    }

    Ident::new(&name, ident.span())
}

// MARK: Tests

#[cfg(test)]
mod tests {
    use proptest::prop_assert_eq;
    use quote::quote;
    use test_strategy::proptest;

    use super::*;

    fn ident(name: &str) -> Ident {
        Ident::new(name, Span::call_site())
    }

    fn arguments() -> MachineArgs {
        syn::parse_str("initial = State::on()").unwrap()
    }

    fn model(item: ItemImpl) -> Model {
        match analyze(arguments(), item) {
            Ok(model) => model,
            Err(error) => panic!("analysis failed: {error}"),
        }
    }

    #[test]
    fn pascal_case_joins_words() {
        assert_eq!(pascal_case(&ident("door_open")), ident("DoorOpen"));
        assert_eq!(pascal_case(&ident("a")), ident("A"));
    }

    #[proptest]
    fn snake_case_splits_words(#[strategy("[a-z]{1,6}(_[a-z]{1,6}){0,3}")] name: String) {
        prop_assert_eq!(snake_case(&pascal_case(&ident(&name))), ident(&name));
    }

    #[test]
    fn state_inputs_are_classified() {
        let model = model(parse_quote! {
            impl Switch {
                #[state]
                fn on(&mut self, event: &Event) -> Outcome<State> {
                    Handled
                }

                #[state]
                fn off(event: &Event) -> Outcome<State> {
                    Handled
                }
            }
        });

        assert_eq!(
            model.states[0].inputs,
            [InputKind::Receiver, InputKind::Event]
        );
        assert_eq!(model.states[1].inputs, [InputKind::Event]);
    }

    #[test]
    fn event_type_is_inferred_from_handlers() {
        let model = model(parse_quote! {
            impl Switch {
                #[state]
                fn on(&mut self) -> Outcome<State> {
                    Handled
                }

                #[state]
                fn off(&mut self, event: &Event) -> Outcome<State> {
                    Handled
                }
            }
        });

        let event_type = model.event_type;

        assert_eq!(quote!(#event_type).to_string(), "Event");
    }

    #[test]
    fn event_type_defaults_to_unit() {
        let model = model(parse_quote! {
            impl Ticker {
                #[state]
                fn on(&mut self) -> Outcome<State> {
                    Handled
                }
            }
        });

        let event_type = model.event_type;

        assert_eq!(quote!(#event_type).to_string(), "()");
    }

    #[test]
    fn missing_initial_is_reported() {
        let arguments: MachineArgs = syn::parse_str("").unwrap();

        let item: ItemImpl = parse_quote! {
            impl Switch {
                #[state]
                fn on(&mut self) -> Outcome<State> {
                    Handled
                }
            }
        };

        let message = match analyze(arguments, item) {
            Ok(_) => panic!("a machine without `initial` should be rejected"),
            Err(error) => error.to_string(),
        };

        assert!(
            message.starts_with("no initial state defined"),
            "unexpected message: {message}"
        );
    }

    #[test]
    fn states_keep_declaration_order() {
        let model = model(parse_quote! {
            impl Switch {
                #[state]
                fn b(&mut self) -> Outcome<State> {
                    Handled
                }

                #[state]
                fn a(&mut self) -> Outcome<State> {
                    Handled
                }
            }
        });

        let variants: Vec<String> = model
            .states
            .iter()
            .map(|state| state.variant.to_string())
            .collect();

        assert_eq!(variants, ["B", "A"]);
    }

    #[test]
    fn sotu_attributes_are_removed_from_item() {
        let model = model(parse_quote! {
            impl Switch {
                /// Documented.
                #[inline]
                #[state]
                fn on(&mut self) -> Outcome<State> {
                    Handled
                }
            }
        });

        let ImplItem::Fn(function) = &model.item.items[0] else {
            panic!("the handler should stay a function");
        };

        let is_state = |attribute: &Attribute| {
            attribute
                .path()
                .segments
                .last()
                .is_some_and(|segment| segment.ident == "state")
        };

        assert!(!function.attrs.iter().any(is_state));
        assert!(
            function
                .attrs
                .iter()
                .any(|attribute| attribute.path().is_ident("inline"))
        );
        assert!(
            function
                .attrs
                .iter()
                .any(|attribute| attribute.path().is_ident("doc"))
        );
    }
}
