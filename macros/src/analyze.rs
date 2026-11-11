//! Validate and classify a `#[state_machine]` impl block.

use proc_macro2::Span;
use syn::ext::IdentExt;
use syn::{
    Attribute, Expr, FnArg, Generics, Ident, ImplItem, ImplItemFn, ItemImpl, Meta, Pat, PatType,
    Path, Type, Visibility, parse_quote,
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
    /// The context type of the machine, which may use the lifetimes `'ctx` and `'data`.
    pub(crate) context_type: Type,
    /// The name of the event input.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "generated parameters have fixed names")
    )]
    pub(crate) event_identifier: Ident,
    /// The name of the context input.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "generated parameters have fixed names")
    )]
    pub(crate) context_identifier: Ident,
    /// The type of the exit points through which the machine can be left.
    pub(crate) exit_type: Type,
    /// The functions that observe dispatch and transitions.
    pub(crate) hooks: HooksModel,
    /// The states in declaration order.
    pub(crate) states: Vec<StateModel>,
    /// The superstates in declaration order.
    pub(crate) superstates: Vec<SuperstateModel>,
    /// The entry and exit actions in declaration order.
    pub(crate) actions: Vec<ActionModel>,
    /// The impl block with the attributes of `sotu` removed.
    pub(crate) item: ItemImpl,
    /// The generics and where-clause of the impl block, which every generated
    /// trait impl declares.
    pub(crate) generics: Generics,
    /// The visibility of the generated types and constructors.
    pub(crate) visibility: Visibility,
    /// The options of the state enum.
    pub(crate) state_type: TypeOptions,
    /// The options of the superstate enum.
    pub(crate) superstate_type: TypeOptions,
}

/// The name and derives of a node enum, and whether the user defines it.
pub(crate) struct TypeOptions {
    /// The name of the enum.
    pub(crate) name: Ident,
    /// Whether the user defines the enum, which the macro then does not generate.
    ///
    /// Always `false` for the superstate enum.
    pub(crate) custom: bool,
    /// The traits to derive on the generated enum.
    pub(crate) derives: Vec<Path>,
}

/// The hook functions of the machine, each `None` when its key is absent.
#[derive(Clone)]
pub(crate) struct HooksModel {
    /// The function called just before a node handles an event.
    pub(crate) before_dispatch: Option<Path>,
    /// The function called just after a node handled an event.
    pub(crate) after_dispatch: Option<Path>,
    /// The function called just before a transition runs its exit actions.
    pub(crate) before_transition: Option<Path>,
    /// The function called just after a transition ran its entry actions.
    pub(crate) after_transition: Option<Path>,
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
    /// The event input, named `event` unless `event_identifier` renames it.
    Event,
    /// The context input, named `context` unless `context_identifier` renames it.
    Context,
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

/// The names that classify the event and context inputs.
///
/// Classification is by name only, so a renamed input replaces the default
/// name, which then classifies as local storage.
struct InputNames {
    /// The name of the event input.
    event: Ident,
    /// The name of the context input.
    context: Ident,
}

/// The handlers and actions of an impl block, in declaration order.
struct Nodes {
    /// The state handlers.
    states: Vec<StateModel>,
    /// The superstate handlers.
    superstates: Vec<SuperstateModel>,
    /// The entry and exit actions.
    actions: Vec<ActionModel>,
    /// The type behind the first event input.
    event_type: Option<Type>,
}

/// The kind of function whose signature a check examines.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
enum Role {
    /// A `#[state]` handler.
    State,
    /// A `#[superstate]` handler.
    Superstate,
    /// An `#[action]`.
    Action,
}

// MARK: Analysis

/// Validate `item` and classify its handlers into a [`Model`].
///
/// A missing `initial` and the errors found in `item` are combined into one
/// result. The inputs of a handler are not checked when its `#[state(...)]`
/// attribute is invalid, because the item is then not classified as a state.
pub(crate) fn analyze(arguments: MachineArgs, mut item: ItemImpl) -> syn::Result<Model> {
    let names = InputNames {
        event: arguments
            .event_identifier
            .unwrap_or_else(|| Ident::new("event", Span::call_site())),
        context: arguments
            .context_identifier
            .unwrap_or_else(|| Ident::new("context", Span::call_site())),
    };

    let (
        Nodes {
            states,
            superstates,
            actions,
            event_type,
        },
        mut errors,
    ) = collect_nodes(&mut item, &names);

    check_duplicate_variants(&states, &superstates, &mut errors);
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
        context_type: arguments.context.unwrap_or_else(|| parse_quote!(())),
        event_identifier: names.event,
        context_identifier: names.context,
        exit_type: arguments
            .exit
            .unwrap_or_else(|| parse_quote!(::core::convert::Infallible)),
        hooks: HooksModel {
            before_dispatch: arguments.before_dispatch,
            after_dispatch: arguments.after_dispatch,
            before_transition: arguments.before_transition,
            after_transition: arguments.after_transition,
        },
        states,
        superstates,
        actions,
        generics: item.generics.clone(),
        item,
        visibility: arguments.visibility.unwrap_or_else(|| parse_quote!(pub)),
        state_type: TypeOptions {
            name: arguments
                .state
                .name
                .unwrap_or_else(|| Ident::new("State", Span::call_site())),
            custom: arguments.state.custom,
            derives: arguments.state.derives,
        },
        superstate_type: TypeOptions {
            name: arguments
                .superstate
                .name
                .unwrap_or_else(|| Ident::new("Superstate", Span::call_site())),
            custom: false,
            derives: arguments.superstate.derives,
        },
    })
}

/// Classify the handlers and actions of `item`, removing the attributes of `sotu`.
///
/// Returns the classified items together with the errors found while
/// classifying them.
fn collect_nodes(item: &mut ItemImpl, names: &InputNames) -> (Nodes, Option<syn::Error>) {
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
                check_signature(function, Role::State, names, &mut errors);
                states.push(analyze_state(
                    function,
                    state_arguments,
                    names,
                    &mut event_type,
                    &mut errors,
                ));
            }
            (ItemAttribute::Superstate(superstate_arguments), ImplItem::Fn(function)) => {
                check_signature(function, Role::Superstate, names, &mut errors);
                superstates.push(analyze_superstate(
                    function,
                    superstate_arguments,
                    names,
                    &mut event_type,
                    &mut errors,
                ));
            }
            (ItemAttribute::Action, ImplItem::Fn(function)) => {
                check_signature(function, Role::Action, names, &mut errors);
                actions.push(analyze_action(function, names));
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

    let nodes = Nodes {
        states,
        superstates,
        actions,
        event_type,
    };

    (nodes, errors)
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

/// Report every variant that repeats the name of an earlier variant of the same enum.
///
/// States and superstates become variants of separate enums, so a state and a
/// superstate may share a variant name.
fn check_duplicate_variants(
    states: &[StateModel],
    superstates: &[SuperstateModel],
    errors: &mut Option<syn::Error>,
) {
    let state_variants: Vec<&Ident> = states.iter().map(|state| &state.variant).collect();
    let superstate_variants: Vec<&Ident> = superstates
        .iter()
        .map(|superstate| &superstate.variant)
        .collect();

    for variants in [state_variants, superstate_variants] {
        for (index, variant) in variants.iter().enumerate() {
            if variants[..index].contains(variant) {
                push_error(
                    errors,
                    syn::Error::new_spanned(variant, format!("duplicate variant `{variant}`")),
                );
            }
        }
    }
}

/// Report the signature errors of the handler or action `function`.
fn check_signature(
    function: &ImplItemFn,
    role: Role,
    names: &InputNames,
    errors: &mut Option<syn::Error>,
) {
    check_generics(function, role, errors);
    check_default_attributes(function, role, names, errors);
    check_input_patterns(function, errors);
    check_inputs_by_reference(function, role, names, errors);
}

/// Report generic parameters declared by `function`.
///
/// The generated code calls handlers and actions without naming generic
/// arguments, so only the generics of the impl block are available to them. A
/// `where` clause declares no parameters and is accepted.
fn check_generics(function: &ImplItemFn, role: Role, errors: &mut Option<syn::Error>) {
    let generics = &function.sig.generics;

    if generics.params.is_empty() {
        return;
    }

    let kind = match role {
        Role::State => "state handlers",
        Role::Superstate => "superstate handlers",
        Role::Action => "actions",
    };

    push_error(
        errors,
        syn::Error::new_spanned(
            generics,
            format!("{kind} can not define their generics themselves"),
        ),
    );
}

/// Report every `#[default]` attribute of `function` that is not on a state input.
///
/// Only the local storage inputs of a state handler declare fields that a
/// constructor initializes, so a default anywhere else has no effect.
fn check_default_attributes(
    function: &ImplItemFn,
    role: Role,
    names: &InputNames,
    errors: &mut Option<syn::Error>,
) {
    for input in &function.sig.inputs {
        let attributes = match input {
            FnArg::Receiver(receiver) => &receiver.attrs,
            FnArg::Typed(typed)
                if role != Role::State
                    || is_named(typed, &names.event)
                    || is_named(typed, &names.context) =>
            {
                &typed.attrs
            }
            FnArg::Typed(_) => continue,
        };

        for attribute in attributes {
            if attribute.path().is_ident("default") {
                push_error(
                    errors,
                    syn::Error::new_spanned(
                        attribute,
                        "`#[default]` is only supported on state inputs",
                    ),
                );
            }
        }
    }
}

/// Report every input of `function` whose pattern is not a plain identifier.
///
/// An input names the local storage field it refers to, which requires an
/// identifier.
fn check_input_patterns(function: &ImplItemFn, errors: &mut Option<syn::Error>) {
    for input in &function.sig.inputs {
        let FnArg::Typed(typed) = input else {
            continue;
        };

        let kind = match &*typed.pat {
            Pat::Ident(_) => continue,
            Pat::Tuple(_) => "tuple",
            Pat::TupleStruct(_) => "tuple struct",
            Pat::Struct(_) => "struct",
            Pat::Slice(_) => "slice",
            Pat::Reference(_) => "reference",
            Pat::Wild(_) => "wildcard",
            Pat::Lit(_) => "literal",
            Pat::Range(_) => "range",
            Pat::Or(_) => "or",
            Pat::Paren(_) => "parenthesized",
            Pat::Path(_) => "path",
            Pat::Rest(_) => "rest",
            Pat::Macro(_) => "macro",
            _ => "this",
        };

        push_error(
            errors,
            syn::Error::new_spanned(&typed.pat, format!("{kind} pattern is not supported")),
        );
    }
}

/// Report every local storage input of `function` that is not a reference.
///
/// Local storage lives in the state, and handlers and actions borrow it, so an
/// input taking a field by value would have to move it out of the state.
/// Inputs whose pattern is not an identifier are left to
/// [`check_input_patterns`].
fn check_inputs_by_reference(
    function: &ImplItemFn,
    role: Role,
    names: &InputNames,
    errors: &mut Option<syn::Error>,
) {
    for input in &function.sig.inputs {
        let FnArg::Typed(typed) = input else {
            continue;
        };

        let is_local_storage = matches!(&*typed.pat, Pat::Ident(_))
            && !is_named(typed, &names.context)
            && (role == Role::Action || !is_named(typed, &names.event));

        if is_local_storage && !matches!(&*typed.ty, Type::Reference(_)) {
            push_error(
                errors,
                syn::Error::new_spanned(&typed.ty, "input must be passed as a reference"),
            );
        }
    }
}

/// Classify the inputs of a handler.
///
/// Only state handlers have `#[default]` attributes taken from their inputs, so
/// `takes_defaults` leaves the attribute in place on a superstate handler, where
/// [`check_default_attributes`] reports it. Inputs that are not local storage
/// are skipped, since [`check_input_patterns`] and [`check_inputs_by_reference`]
/// report them.
fn handler_inputs(
    function: &mut ImplItemFn,
    takes_defaults: bool,
    names: &InputNames,
    event_type: &mut Option<Type>,
    errors: &mut Option<syn::Error>,
) -> Vec<InputKind> {
    let mut inputs = Vec::new();

    for input in &mut function.sig.inputs {
        match input {
            FnArg::Receiver(_) => inputs.push(InputKind::Receiver),
            FnArg::Typed(typed) if is_named(typed, &names.context) => {
                inputs.push(InputKind::Context);
            }
            FnArg::Typed(typed) if is_named(typed, &names.event) => match event_referent(typed) {
                Ok(referent) => {
                    inputs.push(InputKind::Event);
                    event_type.get_or_insert_with(|| referent.clone());
                }
                Err(error) => push_error(errors, error),
            },
            FnArg::Typed(typed) => {
                let Some((name, ty, mutable)) = local_storage_input(typed) else {
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
    names: &InputNames,
    event_type: &mut Option<Type>,
    errors: &mut Option<syn::Error>,
) -> SuperstateModel {
    let handler = function.sig.ident.clone();
    let variant = arguments.name.unwrap_or_else(|| pascal_case(&handler));
    let constructor = snake_case(&variant);
    let inputs = handler_inputs(function, false, names, event_type, errors);

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
    names: &InputNames,
    event_type: &mut Option<Type>,
    errors: &mut Option<syn::Error>,
) -> StateModel {
    let handler = function.sig.ident.clone();
    let variant = arguments.name.unwrap_or_else(|| pascal_case(&handler));
    let constructor = snake_case(&variant);
    let inputs = handler_inputs(function, true, names, event_type, errors);

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

/// Classify the inputs of the action `function`.
///
/// Inputs that are not local storage are skipped, since
/// [`check_input_patterns`] and [`check_inputs_by_reference`] report them.
fn analyze_action(function: &ImplItemFn, names: &InputNames) -> ActionModel {
    let mut inputs = Vec::new();

    for input in &function.sig.inputs {
        match input {
            FnArg::Receiver(_) => inputs.push(InputKind::Receiver),
            FnArg::Typed(typed) if is_named(typed, &names.context) => {
                inputs.push(InputKind::Context);
            }
            FnArg::Typed(typed) => {
                if let Some((name, ty, mutable)) = local_storage_input(typed) {
                    inputs.push(InputKind::LocalStorage {
                        name,
                        ty,
                        mutable,
                        default: None,
                    });
                }
            }
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
        InputKind::Receiver | InputKind::Event | InputKind::Context => None,
    }
}

/// Return whether `typed` is the input called `name`.
fn is_named(typed: &PatType, name: &Ident) -> bool {
    matches!(&*typed.pat, Pat::Ident(pattern) if pattern.ident == *name)
}

/// Return the type behind the reference of the event input `typed`.
///
/// Fails if the event input is not a shared reference.
fn event_referent(typed: &PatType) -> syn::Result<&Type> {
    match &*typed.ty {
        Type::Reference(reference) if reference.mutability.is_none() => Ok(&reference.elem),
        ty => Err(syn::Error::new_spanned(
            ty,
            "the event input must be a shared reference, as in `event: &Event`",
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

    fn names() -> InputNames {
        InputNames {
            event: ident("event"),
            context: ident("context"),
        }
    }

    fn nodes(mut item: ItemImpl) -> Nodes {
        match collect_nodes(&mut item, &names()) {
            (nodes, None) => nodes,
            (_, Some(error)) => panic!("classification failed: {error}"),
        }
    }

    fn messages(errors: Option<syn::Error>) -> Vec<String> {
        errors
            .into_iter()
            .flat_map(syn::Error::into_iter)
            .map(|error| error.to_string())
            .collect()
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
    fn context_input_is_classified() {
        let model = model(parse_quote! {
            impl Writer {
                #[state]
                fn on(context: &mut Sink<'_, '_>, event: &Event) -> Outcome<State> {
                    Handled
                }
            }
        });

        assert_eq!(
            model.states[0].inputs,
            [InputKind::Context, InputKind::Event]
        );
    }

    #[test]
    fn renamed_identifiers_classify_inputs() {
        let arguments: MachineArgs = syn::parse_str(
            "initial = State::on(), event_identifier = input, context_identifier = sink",
        )
        .unwrap();

        let item: ItemImpl = parse_quote! {
            impl Writer {
                #[state]
                fn on(input: &Event, sink: &mut Sink<'_, '_>) -> Outcome<State> {
                    Handled
                }
            }
        };

        let model = match analyze(arguments, item) {
            Ok(model) => model,
            Err(error) => panic!("analysis failed: {error}"),
        };

        assert_eq!(
            model.states[0].inputs,
            [InputKind::Event, InputKind::Context]
        );
        assert_eq!(model.event_identifier, ident("input"));
        assert_eq!(model.context_identifier, ident("sink"));
    }

    #[test]
    fn context_type_defaults_to_unit() {
        let model = model(parse_quote! {
            impl Ticker {
                #[state]
                fn on(&mut self) -> Outcome<State> {
                    Handled
                }
            }
        });

        let context_type = model.context_type;

        assert_eq!(quote!(#context_type).to_string(), "()");
    }

    #[test]
    fn exit_type_defaults_to_infallible() {
        let model = model(parse_quote! {
            impl Ticker {
                #[state]
                fn on(&mut self) -> Outcome<State> {
                    Handled
                }
            }
        });

        let exit_type = model.exit_type;

        assert_eq!(
            quote!(#exit_type).to_string(),
            ":: core :: convert :: Infallible"
        );
    }

    #[test]
    fn type_options_default_to_state_and_superstate() {
        let model = model(parse_quote! {
            impl Ticker {
                #[state]
                fn on(&mut self) -> Outcome<State> {
                    Handled
                }
            }
        });

        let visibility = model.visibility;

        assert_eq!(quote!(#visibility).to_string(), "pub");
        assert_eq!(model.state_type.name, ident("State"));
        assert!(!model.state_type.custom);
        assert!(model.state_type.derives.is_empty());
        assert_eq!(model.superstate_type.name, ident("Superstate"));
        assert!(!model.superstate_type.custom);
        assert!(model.superstate_type.derives.is_empty());
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
    fn duplicate_variant_is_reported() {
        let nodes = nodes(parse_quote! {
            impl Lamp {
                #[state]
                fn idle() -> Outcome<State> {
                    Handled
                }

                #[state(name = Idle)]
                fn waiting() -> Outcome<State> {
                    Handled
                }
            }
        });

        let mut errors = None;

        check_duplicate_variants(&nodes.states, &nodes.superstates, &mut errors);

        assert_eq!(messages(errors), ["duplicate variant `Idle`"]);
    }

    #[test]
    fn handler_generics_are_reported() {
        let state: ImplItemFn = parse_quote! {
            fn on<T>(event: &Event) -> Outcome<State> {
                Handled
            }
        };

        let superstate: ImplItemFn = parse_quote! {
            fn group<'a>() -> Outcome<State> {
                Super
            }
        };

        let mut errors = None;

        check_generics(&state, Role::State, &mut errors);
        check_generics(&superstate, Role::Superstate, &mut errors);

        assert_eq!(
            messages(errors),
            [
                "state handlers can not define their generics themselves",
                "superstate handlers can not define their generics themselves",
            ]
        );
    }

    #[test]
    fn action_generics_are_reported() {
        let action: ImplItemFn = parse_quote! {
            fn prepare<const N: usize>(counter: &mut u32) {}
        };

        let plain: ImplItemFn = parse_quote! {
            fn reset(counter: &mut u32) where Self: Sized {}
        };

        let mut errors = None;

        check_generics(&action, Role::Action, &mut errors);
        check_generics(&plain, Role::Action, &mut errors);

        assert_eq!(
            messages(errors),
            ["actions can not define their generics themselves"]
        );
    }

    #[test]
    fn default_on_non_state_input_is_reported() {
        let state: ImplItemFn = parse_quote! {
            fn on(#[default] event: &Event, #[default] counter: &mut u32) -> Outcome<State> {
                Handled
            }
        };

        let superstate: ImplItemFn = parse_quote! {
            fn group(#[default] counter: &u32) -> Outcome<State> {
                Super
            }
        };

        let mut errors = None;

        check_default_attributes(&state, Role::State, &names(), &mut errors);
        check_default_attributes(&superstate, Role::Superstate, &names(), &mut errors);

        assert_eq!(
            messages(errors),
            [
                "`#[default]` is only supported on state inputs",
                "`#[default]` is only supported on state inputs",
            ]
        );
    }

    #[test]
    fn non_identifier_pattern_is_reported() {
        let state: ImplItemFn = parse_quote! {
            fn on((a, b): &(u8, u8), _: &u8, counter: &u32) -> Outcome<State> {
                Handled
            }
        };

        let mut errors = None;

        check_input_patterns(&state, &mut errors);

        assert_eq!(
            messages(errors),
            [
                "tuple pattern is not supported",
                "wildcard pattern is not supported"
            ]
        );
    }

    #[test]
    fn input_by_value_is_reported() {
        let state: ImplItemFn = parse_quote! {
            fn on(&mut self, counter: u32, event: &Event, total: &u32) -> Outcome<State> {
                Handled
            }
        };

        let mut errors = None;

        check_inputs_by_reference(&state, Role::State, &names(), &mut errors);

        assert_eq!(messages(errors), ["input must be passed as a reference"]);
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
