//! Validate and classify a `#[state_machine]` impl block.

use proc_macro2::Span;
use syn::ext::IdentExt;
use syn::{
    Attribute, Expr, FnArg, Ident, ImplItem, ImplItemFn, ItemImpl, Meta, Pat, PatType, Type,
    parse_quote,
};

use crate::parse::{
    ItemAttribute, MachineArgs, StateArgs, SuperstateArgs, parse_item_attribute, push_error,
};

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
    /// The superstates in declaration order.
    pub(crate) superstates: Vec<SuperstateModel>,
    /// The entry and exit actions in declaration order.
    pub(crate) actions: Vec<ActionModel>,
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
    /// The handler name of the enclosing superstate.
    pub(crate) superstate: Option<Ident>,
    /// The classified inputs of the handler, in declaration order.
    pub(crate) inputs: Vec<InputKind>,
    /// The local storage fields: the handler's inputs, then the `local_storage` key.
    pub(crate) fields: Vec<FieldModel>,
    /// The action to run when the state is entered.
    pub(crate) entry_action: Option<Ident>,
    /// The action to run when the state is exited.
    pub(crate) exit_action: Option<Ident>,
    /// The doc comments of the handler.
    pub(crate) docs: Vec<Attribute>,
}

/// A superstate handler of the machine.
pub(crate) struct SuperstateModel {
    /// The name of the handler function.
    pub(crate) handler: Ident,
    /// The name of the generated enum variant.
    pub(crate) variant: Ident,
    /// The name of the generated `initial` constructor.
    pub(crate) constructor: Ident,
    /// The handler name of the enclosing superstate.
    pub(crate) superstate: Option<Ident>,
    /// The classified inputs of the handler, in declaration order.
    pub(crate) inputs: Vec<InputKind>,
    /// The borrowed fields, such as `&'sub mut u32`: the handler's inputs, then
    /// the `local_storage` key.
    pub(crate) fields: Vec<FieldModel>,
    /// The action to run when the superstate is entered.
    pub(crate) entry_action: Option<Ident>,
    /// The action to run when the superstate is exited.
    pub(crate) exit_action: Option<Ident>,
    /// The state expression that the generated `initial` constructor returns.
    pub(crate) initial: Option<Expr>,
    /// The doc comments of the handler.
    pub(crate) docs: Vec<Attribute>,
}

/// A local storage field of a state.
pub(crate) struct FieldModel {
    /// The name of the field.
    pub(crate) name: Ident,
    /// The type of the field.
    pub(crate) ty: Type,
    /// The value the constructor gives the field instead of taking a parameter.
    pub(crate) default: Option<DefaultValue>,
}

/// An entry or exit action of the machine.
pub(crate) struct ActionModel {
    /// The name of the action function.
    pub(crate) name: Ident,
    /// The classified inputs of the action, in declaration order.
    pub(crate) inputs: Vec<InputKind>,
}

/// What an input of a handler or action receives.
// Inputs live in short vectors for the duration of one macro expansion, so the
// size of the largest variant does not justify boxing the syntax trees.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Eq, PartialEq, Debug)]
pub(crate) enum InputKind {
    /// The receiver, which is the shared storage.
    Receiver,
    /// The input named `event`.
    Event,
    /// A reference to the local storage field called `name`.
    LocalStorage {
        /// The name of the field.
        name: Ident,
        /// The type behind the reference.
        ty: Type,
        /// Whether the reference is mutable.
        mutable: bool,
        /// The value of a `#[default]` attribute on the input.
        default: Option<DefaultValue>,
    },
}

/// The value given by a `#[default]` attribute.
#[derive(Clone, Eq, PartialEq, Debug)]
pub(crate) enum DefaultValue {
    /// `#[default]`, which is the `Default` value of the field's type.
    Default,
    /// `#[default = expression]`.
    Expr(Expr),
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
    let mut superstates = Vec::new();
    let mut actions = Vec::new();
    let mut event_type = None;

    for impl_item in &mut item.items {
        let Some(attribute) = take_item_attribute(impl_item, &mut errors) else {
            continue;
        };

        match (attribute, &mut *impl_item) {
            (ItemAttribute::State(state_arguments), ImplItem::Fn(function)) => {
                states.push(analyze_state(
                    function,
                    state_arguments,
                    &mut event_type,
                    &mut errors,
                ));
            }
            (ItemAttribute::Superstate(superstate_arguments), ImplItem::Fn(function)) => {
                superstates.push(analyze_superstate(
                    function,
                    superstate_arguments,
                    &mut event_type,
                    &mut errors,
                ));
            }
            (ItemAttribute::Action, ImplItem::Fn(function)) => {
                actions.push(analyze_action(function, &mut errors));
            }
            (ItemAttribute::State(_), _) => push_error(
                &mut errors,
                syn::Error::new_spanned(&*impl_item, "`#[state]` can only be applied to functions"),
            ),
            (ItemAttribute::Superstate(_), _) => push_error(
                &mut errors,
                syn::Error::new_spanned(
                    &*impl_item,
                    "`#[superstate]` can only be applied to functions",
                ),
            ),
            (ItemAttribute::Action, _) => push_error(
                &mut errors,
                syn::Error::new_spanned(
                    &*impl_item,
                    "`#[action]` can only be applied to functions",
                ),
            ),
        }
    }

    check_action_references(&states, &superstates, &actions, &mut errors);
    check_superstate_references(&states, &superstates, &mut errors);

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
        superstates,
        actions,
        item,
    })
}

/// Remove the attributes of `sotu` from `item`, returning the one it carried.
fn take_item_attribute(
    item: &mut ImplItem,
    errors: &mut Option<syn::Error>,
) -> Option<ItemAttribute> {
    let attributes = match item {
        ImplItem::Const(item) => &mut item.attrs,
        ImplItem::Fn(item) => &mut item.attrs,
        ImplItem::Type(item) => &mut item.attrs,
        ImplItem::Macro(item) => &mut item.attrs,
        _ => return None,
    };

    let mut taken = None;

    attributes.retain(|attribute| match parse_item_attribute(attribute) {
        Ok(None) => true,
        Ok(Some(item_attribute)) => {
            taken = Some(item_attribute);

            false
        }
        Err(error) => {
            push_error(errors, error);

            false
        }
    });

    taken
}

/// Report every `entry_action` and `exit_action` that names no `#[action]`.
fn check_action_references(
    states: &[StateModel],
    superstates: &[SuperstateModel],
    actions: &[ActionModel],
    errors: &mut Option<syn::Error>,
) {
    let mut check = |reference: &Option<Ident>, message: &str| {
        if let Some(name) = reference
            && !actions.iter().any(|action| action.name == *name)
        {
            push_error(errors, syn::Error::new_spanned(name, message));
        }
    };

    for state in states {
        check(&state.entry_action, "entry action not found");
        check(&state.exit_action, "exit action not found");
    }

    for superstate in superstates {
        check(&superstate.entry_action, "entry action not found");
        check(&superstate.exit_action, "exit action not found");
    }
}

/// Report every `superstate` link that names no `#[superstate]` handler.
fn check_superstate_references(
    states: &[StateModel],
    superstates: &[SuperstateModel],
    errors: &mut Option<syn::Error>,
) {
    let links = states
        .iter()
        .map(|state| &state.superstate)
        .chain(superstates.iter().map(|superstate| &superstate.superstate));

    for name in links.flatten() {
        if !superstates
            .iter()
            .any(|superstate| superstate.handler == *name)
        {
            push_error(
                errors,
                syn::Error::new_spanned(name, "superstate not found"),
            );
        }
    }
}

/// Classify the inputs of a handler.
///
/// Only state handlers have `#[default]` attributes taken from their inputs, so
/// `takes_defaults` leaves the attribute in place on a superstate handler, where
/// the compiler rejects it.
fn handler_inputs(
    function: &mut ImplItemFn,
    takes_defaults: bool,
    event_type: &mut Option<Type>,
    errors: &mut Option<syn::Error>,
) -> Vec<InputKind> {
    let mut inputs = Vec::new();

    for input in &mut function.sig.inputs {
        match input {
            FnArg::Receiver(_) => inputs.push(InputKind::Receiver),
            FnArg::Typed(typed) if is_event(typed) => match event_referent(typed) {
                Ok(referent) => {
                    inputs.push(InputKind::Event);
                    event_type.get_or_insert_with(|| referent.clone());
                }
                Err(error) => push_error(errors, error),
            },
            FnArg::Typed(typed) => {
                let Some((name, ty, mutable)) = local_storage_input(typed) else {
                    let node = if takes_defaults {
                        "state"
                    } else {
                        "superstate"
                    };

                    push_error(
                        errors,
                        syn::Error::new_spanned(
                            &*typed,
                            format!(
                                "unsupported input; {node} handlers take a receiver, `event` and \
                                 references to local storage"
                            ),
                        ),
                    );

                    continue;
                };

                let default = if takes_defaults {
                    take_default(&mut typed.attrs, errors)
                } else {
                    None
                };

                inputs.push(InputKind::LocalStorage {
                    name,
                    ty,
                    mutable,
                    default,
                });
            }
        }
    }

    inputs
}

fn analyze_superstate(
    function: &mut ImplItemFn,
    arguments: SuperstateArgs,
    event_type: &mut Option<Type>,
    errors: &mut Option<syn::Error>,
) -> SuperstateModel {
    let handler = function.sig.ident.clone();
    let variant = arguments.name.unwrap_or_else(|| pascal_case(&handler));
    let constructor = snake_case(&variant);
    let inputs = handler_inputs(function, false, event_type, errors);

    let mut fields: Vec<FieldModel> = inputs.iter().filter_map(borrowed_field).collect();

    fields.extend(
        arguments
            .local_storage
            .into_iter()
            .map(|(name, ty)| FieldModel {
                name,
                ty,
                default: None,
            }),
    );

    let docs = handler_docs(function);

    SuperstateModel {
        handler,
        variant,
        constructor,
        superstate: arguments.superstate,
        inputs,
        fields,
        entry_action: arguments.entry_action,
        exit_action: arguments.exit_action,
        initial: arguments.initial,
        docs,
    }
}

/// Return the field through which a superstate borrows the input `input`, if it
/// is local storage.
fn borrowed_field(input: &InputKind) -> Option<FieldModel> {
    let InputKind::LocalStorage {
        name, ty, mutable, ..
    } = input
    else {
        return None;
    };

    let ty: Type = if *mutable {
        parse_quote!(&'sub mut #ty)
    } else {
        parse_quote!(&'sub #ty)
    };

    Some(FieldModel {
        name: name.clone(),
        ty,
        default: None,
    })
}

fn handler_docs(function: &ImplItemFn) -> Vec<Attribute> {
    function
        .attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident("doc"))
        .cloned()
        .collect()
}

fn analyze_state(
    function: &mut ImplItemFn,
    arguments: StateArgs,
    event_type: &mut Option<Type>,
    errors: &mut Option<syn::Error>,
) -> StateModel {
    let handler = function.sig.ident.clone();
    let variant = arguments.name.unwrap_or_else(|| pascal_case(&handler));
    let constructor = snake_case(&variant);
    let inputs = handler_inputs(function, true, event_type, errors);

    let mut fields: Vec<FieldModel> = inputs.iter().filter_map(input_field).collect();

    fields.extend(
        arguments
            .local_storage
            .into_iter()
            .map(|(name, ty)| FieldModel {
                name,
                ty,
                default: None,
            }),
    );

    let docs = handler_docs(function);

    StateModel {
        handler,
        variant,
        constructor,
        superstate: arguments.superstate,
        inputs,
        fields,
        entry_action: arguments.entry_action,
        exit_action: arguments.exit_action,
        docs,
    }
}

fn analyze_action(function: &ImplItemFn, errors: &mut Option<syn::Error>) -> ActionModel {
    let mut inputs = Vec::new();

    for input in &function.sig.inputs {
        match input {
            FnArg::Receiver(_) => inputs.push(InputKind::Receiver),
            FnArg::Typed(typed) => match local_storage_input(typed) {
                Some((name, ty, mutable)) => inputs.push(InputKind::LocalStorage {
                    name,
                    ty,
                    mutable,
                    default: None,
                }),
                None => push_error(
                    errors,
                    syn::Error::new_spanned(
                        typed,
                        "unsupported input; actions take a receiver and references to local \
                         storage",
                    ),
                ),
            },
        }
    }

    ActionModel {
        name: function.sig.ident.clone(),
        inputs,
    }
}

/// Return the field name, field type and mutability of the local storage input `typed`.
///
/// Returns `None` unless `typed` is a plain identifier bound to a reference.
fn local_storage_input(typed: &PatType) -> Option<(Ident, Type, bool)> {
    let Pat::Ident(pattern) = &*typed.pat else {
        return None;
    };

    let Type::Reference(reference) = &*typed.ty else {
        return None;
    };

    Some((
        pattern.ident.clone(),
        (*reference.elem).clone(),
        reference.mutability.is_some(),
    ))
}

/// Remove the `#[default]` attributes from `attributes`, returning the value of the last.
fn take_default(
    attributes: &mut Vec<Attribute>,
    errors: &mut Option<syn::Error>,
) -> Option<DefaultValue> {
    let mut default = None;

    attributes.retain(|attribute| {
        if !attribute.path().is_ident("default") {
            return true;
        }

        match &attribute.meta {
            Meta::Path(_) => default = Some(DefaultValue::Default),
            Meta::NameValue(name_value) => {
                default = Some(DefaultValue::Expr(name_value.value.clone()));
            }
            Meta::List(list) => push_error(
                errors,
                syn::Error::new_spanned(list, "expected `#[default]` or `#[default = expression]`"),
            ),
        }

        false
    });

    default
}

/// Return the field that the input `input` declares, if it is local storage.
fn input_field(input: &InputKind) -> Option<FieldModel> {
    match input {
        InputKind::LocalStorage {
            name, ty, default, ..
        } => Some(FieldModel {
            name: name.clone(),
            ty: ty.clone(),
            default: default.clone(),
        }),
        InputKind::Receiver | InputKind::Event => None,
    }
}

/// Return whether `typed` is the input named `event`.
fn is_event(typed: &PatType) -> bool {
    matches!(&*typed.pat, Pat::Ident(pattern) if pattern.ident == "event")
}

/// Return the type behind the reference of the `event` input `typed`.
///
/// Fails if `event` is not a shared reference.
fn event_referent(typed: &PatType) -> syn::Result<&Type> {
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
    fn reference_inputs_become_local_storage() {
        let model = model(parse_quote! {
            impl Counter {
                #[state]
                fn on(&mut self, counter: &mut u32, event: &Event) -> Outcome<State> {
                    Handled
                }
            }
        });

        let state = &model.states[0];

        let [
            InputKind::Receiver,
            InputKind::LocalStorage {
                name,
                ty,
                mutable,
                default,
            },
            InputKind::Event,
        ] = state.inputs.as_slice()
        else {
            panic!("the inputs should be a receiver, local storage and `event`");
        };

        assert_eq!(*name, ident("counter"));
        assert_eq!(quote!(#ty).to_string(), "u32");
        assert!(*mutable);
        assert_eq!(*default, None);

        let [field] = state.fields.as_slice() else {
            panic!("the state should have one field");
        };

        let field_type = &field.ty;

        assert_eq!(field.name, ident("counter"));
        assert_eq!(quote!(#field_type).to_string(), "u32");
    }

    #[test]
    fn default_attributes_are_recorded() {
        let model = model(parse_quote! {
            impl Counter {
                #[state]
                fn on(#[default] lit: &mut bool, #[default = 3] level: &u8) -> Outcome<State> {
                    Handled
                }
            }
        });

        let state = &model.states[0];

        assert_eq!(state.fields[0].default, Some(DefaultValue::Default));

        let Some(DefaultValue::Expr(level)) = &state.fields[1].default else {
            panic!("`level` should have an expression default");
        };

        assert_eq!(quote!(#level).to_string(), "3");

        let ImplItem::Fn(function) = &model.item.items[0] else {
            panic!("the handler should stay a function");
        };

        let emitted = &function.sig;

        assert_eq!(
            quote!(#emitted).to_string(),
            "fn on (lit : & mut bool , level : & u8) -> Outcome < State >"
        );
    }

    #[test]
    fn local_storage_key_appends_fields() {
        let model = model(parse_quote! {
            impl Counter {
                #[state(local_storage(started: bool))]
                fn on(counter: &mut u32, #[default] lit: &mut bool) -> Outcome<State> {
                    Handled
                }
            }
        });

        let fields: Vec<String> = model.states[0]
            .fields
            .iter()
            .map(|field| field.name.to_string())
            .collect();

        assert_eq!(fields, ["counter", "lit", "started"]);
    }

    #[test]
    fn name_override_sets_variant_and_constructor() {
        let model = model(parse_quote! {
            impl Door {
                #[state(name = DoorOpen)]
                fn waiting() -> Outcome<State> {
                    Handled
                }
            }
        });

        let state = &model.states[0];

        assert_eq!(state.handler, ident("waiting"));
        assert_eq!(state.variant, ident("DoorOpen"));
        assert_eq!(state.constructor, ident("door_open"));
    }

    #[test]
    fn unknown_entry_action_is_reported() {
        let item: ItemImpl = parse_quote! {
            impl Door {
                #[state(entry_action = missing)]
                fn waiting() -> Outcome<State> {
                    Handled
                }

                #[action]
                fn present() {}
            }
        };

        let message = match analyze(arguments(), item) {
            Ok(_) => panic!("an unknown entry action should be rejected"),
            Err(error) => error.to_string(),
        };

        assert!(
            message.starts_with("entry action not found"),
            "unexpected message: {message}"
        );
    }

    #[test]
    fn superstate_inputs_become_borrowed_fields() {
        let model = model(parse_quote! {
            impl Counter {
                #[superstate]
                fn blinking(counter: &mut u32, event: &Event) -> Outcome<State> {
                    Super
                }
            }
        });

        let [field] = model.superstates[0].fields.as_slice() else {
            panic!("the superstate should have one field");
        };

        let field_type = &field.ty;

        assert_eq!(field.name, ident("counter"));
        assert_eq!(quote!(#field_type).to_string(), "& 'sub mut u32");
    }

    #[test]
    fn superstate_links_are_resolved() {
        let model = model(parse_quote! {
            impl Counter {
                #[state(superstate = blinking)]
                fn on(event: &Event) -> Outcome<State> {
                    Super
                }

                #[superstate]
                fn blinking(event: &Event) -> Outcome<State> {
                    Super
                }
            }
        });

        assert_eq!(model.states[0].superstate, Some(ident("blinking")));
    }

    #[test]
    fn unknown_superstate_is_reported() {
        let item: ItemImpl = parse_quote! {
            impl Counter {
                #[state(superstate = missing)]
                fn on(event: &Event) -> Outcome<State> {
                    Super
                }

                #[superstate]
                fn blinking(event: &Event) -> Outcome<State> {
                    Super
                }
            }
        };

        let message = match analyze(arguments(), item) {
            Ok(_) => panic!("an unknown superstate should be rejected"),
            Err(error) => error.to_string(),
        };

        assert!(
            message.starts_with("superstate not found"),
            "unexpected message: {message}"
        );
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
