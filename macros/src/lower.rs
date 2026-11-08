//! Resolve every name, type and call the generated code needs.

use proc_macro2::Span;
use syn::{Attribute, Expr, Generics, Ident, ItemImpl, Type, parse_quote};

use crate::analyze::{
    ActionModel, DefaultValue, FieldModel, HooksModel, InputKind, Model, StateModel,
    SuperstateModel,
};

// MARK: Intermediate Representation

/// Everything [`generate`](crate::codegen::generate) emits.
pub(crate) struct Ir {
    /// The impl block with the attributes of `sotu` removed.
    pub(crate) item: ItemImpl,
    /// The generated state enum.
    pub(crate) state_enum: StateEnumIr,
    /// The associated types and `initial` of the machine traits.
    pub(crate) machine: MachineIr,
    /// The implementation of the blocking `State` trait.
    pub(crate) dispatch: DispatchIr,
    /// The generated superstate enum, if the machine has superstates.
    pub(crate) superstate: Option<SuperstateIr>,
}

/// The generated state enum and its constructors.
pub(crate) struct StateEnumIr {
    /// The name of the enum.
    pub(crate) name: Ident,
    /// The variants in declaration order.
    pub(crate) variants: Vec<VariantIr>,
}

/// A variant of the state enum.
pub(crate) struct VariantIr {
    /// The doc comments copied from the handler.
    pub(crate) docs: Vec<Attribute>,
    /// The name of the variant.
    pub(crate) name: Ident,
    /// The name of the constructor.
    pub(crate) constructor: Ident,
    /// The local storage fields in declaration order.
    pub(crate) fields: Vec<FieldIr>,
}

/// A local storage field of a variant.
pub(crate) struct FieldIr {
    /// The name of the field.
    pub(crate) name: Ident,
    /// The type of the field.
    pub(crate) ty: Type,
    /// The value the constructor assigns, or `None` if the field is a constructor parameter.
    pub(crate) default: Option<Expr>,
}

/// The items shared by `IntoStateMachine` and `Submachine`.
pub(crate) struct MachineIr {
    /// The type implementing the traits.
    pub(crate) shared_storage: Type,
    /// The generics of the impl block, which the context type may use.
    pub(crate) generics: Generics,
    /// The event type of the machine.
    pub(crate) event_type: Type,
    /// The context type of the machine, which may use the lifetimes `'ctx` and `'data`.
    pub(crate) context_type: Type,
    /// The type of the exit points of the machine.
    pub(crate) exit_type: Type,
    /// The name of the state enum.
    pub(crate) state: Ident,
    /// The superstate enum, or `None` if the associated type is `()`.
    pub(crate) superstate: Option<SuperstateTypeIr>,
    /// The expression that builds the initial state.
    pub(crate) initial: Expr,
    /// The hook functions, which only `IntoStateMachine` declares.
    pub(crate) hooks: HooksModel,
}

/// The implementation of the blocking `State` trait.
pub(crate) struct DispatchIr {
    /// The type the handlers belong to.
    pub(crate) shared_storage: Type,
    /// The generics of the impl block.
    pub(crate) generics: Generics,
    /// The event type of the machine.
    pub(crate) event_type: Type,
    /// The context type of the machine.
    pub(crate) context_type: Type,
    /// The type of the exit points of the machine.
    pub(crate) exit_type: Type,
    /// The name of the state enum.
    pub(crate) state: Ident,
    /// The name of the shared storage parameter, underscored when unused.
    pub(crate) shared_storage_parameter: Ident,
    /// The name of the event parameter, underscored when unused.
    pub(crate) event_parameter: Ident,
    /// The name of the context parameter, underscored when unused.
    pub(crate) context_parameter: Ident,
    /// One match arm per variant, in declaration order.
    pub(crate) arms: Vec<ArmIr>,
    /// The body of `call_entry_action`, if any state has an entry action.
    pub(crate) entry_actions: Option<ActionDispatchIr>,
    /// The body of `call_exit_action`, if any state has an exit action.
    pub(crate) exit_actions: Option<ActionDispatchIr>,
    /// The body of `superstate`, if the machine has superstates.
    pub(crate) parents: Option<ParentDispatchIr>,
}

/// The name of the superstate enum and whether it takes the lifetime `'sub`.
///
/// An enum without borrowed fields can not declare the lifetime, because an
/// unused lifetime parameter is a compile error.
#[derive(Clone)]
pub(crate) struct SuperstateTypeIr {
    /// The name of the enum.
    pub(crate) name: Ident,
    /// Whether any variant has a borrowed field.
    pub(crate) has_lifetime: bool,
}

/// The generated superstate enum and the implementation of the blocking
/// `Superstate` trait.
pub(crate) struct SuperstateIr {
    /// The type of the enum.
    pub(crate) ty: SuperstateTypeIr,
    /// The variants in declaration order.
    pub(crate) variants: Vec<SuperstateVariantIr>,
    /// The type the handlers belong to.
    pub(crate) shared_storage: Type,
    /// The generics of the impl block.
    pub(crate) generics: Generics,
    /// The event type of the machine.
    pub(crate) event_type: Type,
    /// The context type of the machine.
    pub(crate) context_type: Type,
    /// The type of the exit points of the machine.
    pub(crate) exit_type: Type,
    /// The name of the state enum.
    pub(crate) state: Ident,
    /// The name of the shared storage parameter, underscored when unused.
    pub(crate) shared_storage_parameter: Ident,
    /// The name of the event parameter, underscored when unused.
    pub(crate) event_parameter: Ident,
    /// The name of the context parameter, underscored when unused.
    pub(crate) context_parameter: Ident,
    /// One match arm per variant, in declaration order.
    pub(crate) arms: Vec<ArmIr>,
    /// The body of `call_entry_action`, if any superstate has an entry action.
    pub(crate) entry_actions: Option<ActionDispatchIr>,
    /// The body of `call_exit_action`, if any superstate has an exit action.
    pub(crate) exit_actions: Option<ActionDispatchIr>,
    /// The body of `superstate`.
    pub(crate) parents: ParentDispatchIr,
}

/// A variant of the superstate enum.
pub(crate) struct SuperstateVariantIr {
    /// The doc comments copied from the handler.
    pub(crate) docs: Vec<Attribute>,
    /// The name of the variant.
    pub(crate) name: Ident,
    /// The name of the `initial` constructor.
    pub(crate) constructor: Ident,
    /// The expression the `initial` constructor returns, if the superstate has one.
    pub(crate) initial: Option<Expr>,
    /// The borrowed fields in declaration order.
    pub(crate) fields: Vec<FieldIr>,
}

/// A match arm that calls one handler.
pub(crate) struct ArmIr {
    /// The pattern the arm matches.
    pub(crate) pattern: PatternIr,
    /// The call to the handler.
    pub(crate) call: CallIr,
}

/// The method that runs the entry or exit action of the active node.
pub(crate) struct ActionDispatchIr {
    /// The name of the shared storage parameter, underscored when unused.
    pub(crate) shared_storage_parameter: Ident,
    /// The name of the context parameter, underscored when unused.
    pub(crate) context_parameter: Ident,
    /// One match arm per variant, in declaration order.
    pub(crate) arms: Vec<ActionArmIr>,
}

/// A match arm that runs the action of one node, if it has one.
pub(crate) struct ActionArmIr {
    /// The pattern the arm matches.
    pub(crate) pattern: PatternIr,
    /// The call to the action.
    pub(crate) call: Option<CallIr>,
}

/// The method that builds the superstate enclosing a node.
pub(crate) struct ParentDispatchIr {
    /// The type of the superstate enum.
    pub(crate) superstate: SuperstateTypeIr,
    /// One match arm per variant, in declaration order.
    pub(crate) arms: Vec<ParentArmIr>,
}

/// A match arm that builds the parent of one node.
pub(crate) struct ParentArmIr {
    /// The pattern the arm matches, binding the fields the parent borrows.
    pub(crate) pattern: PatternIr,
    /// The parent, or `None` for a top-level node.
    pub(crate) parent: Option<ParentIr>,
}

/// A superstate built from the fields of its child.
pub(crate) struct ParentIr {
    /// The variant of the superstate enum.
    pub(crate) variant: Ident,
    /// The fields of the variant.
    pub(crate) fields: Vec<ParentFieldIr>,
}

/// A borrowed field of a parent.
pub(crate) struct ParentFieldIr {
    /// The name of the field.
    pub(crate) name: Ident,
    /// The value of the field, or `None` if the binding of the same name is the value.
    pub(crate) value: Option<Expr>,
}

/// The pattern of one variant, binding only the fields a call takes.
///
/// Binding the others would leave them unused in the generated code.
pub(crate) struct PatternIr {
    /// The variant the pattern matches.
    pub(crate) variant: Ident,
    /// The fields the pattern binds, in the order the callee declares them.
    pub(crate) bindings: Vec<Ident>,
    /// Whether the pattern skips any field of the variant.
    pub(crate) has_rest: bool,
}

/// A call to a function of the impl block.
pub(crate) struct CallIr {
    /// The function to call.
    pub(crate) function: Ident,
    /// The arguments of the call, in the order the function declares its inputs.
    pub(crate) arguments: Vec<Ident>,
}

/// The parts of a state or superstate that action dispatch reads.
struct ActionNode<'model> {
    variant: &'model Ident,
    fields: &'model [FieldModel],
    action: Option<&'model Ident>,
}

// MARK: Lowering

const SHARED_STORAGE: &str = "shared_storage";
const EVENT: &str = "event";
const CONTEXT: &str = "context";

/// Resolve `model` into the [`Ir`] that code generation reads.
pub(crate) fn lower(model: &Model) -> Ir {
    let state = Ident::new("State", Span::call_site());
    let superstate_type = lower_superstate_type(model);

    Ir {
        item: model.item.clone(),
        state_enum: StateEnumIr {
            name: state.clone(),
            variants: model.states.iter().map(lower_variant).collect(),
        },
        machine: MachineIr {
            shared_storage: model.shared_storage.clone(),
            generics: model.item.generics.clone(),
            event_type: model.event_type.clone(),
            context_type: model.context_type.clone(),
            exit_type: model.exit_type.clone(),
            state: state.clone(),
            superstate: superstate_type.clone(),
            initial: model.initial.clone(),
            hooks: model.hooks.clone(),
        },
        dispatch: DispatchIr {
            shared_storage: model.shared_storage.clone(),
            generics: model.item.generics.clone(),
            event_type: model.event_type.clone(),
            context_type: model.context_type.clone(),
            exit_type: model.exit_type.clone(),
            state: state.clone(),
            shared_storage_parameter: handler_parameter(
                model,
                &InputKind::Receiver,
                SHARED_STORAGE,
            ),
            event_parameter: handler_parameter(model, &InputKind::Event, EVENT),
            context_parameter: handler_parameter(model, &InputKind::Context, CONTEXT),
            arms: model
                .states
                .iter()
                .map(|state| {
                    lower_arm(&state.variant, &state.fields, &state.handler, &state.inputs)
                })
                .collect(),
            entry_actions: lower_state_actions(model, |state| state.entry_action.as_ref()),
            exit_actions: lower_state_actions(model, |state| state.exit_action.as_ref()),
            parents: superstate_type
                .clone()
                .map(|superstate| lower_state_parents(model, superstate)),
        },
        superstate: superstate_type.map(|ty| lower_superstate(model, ty, state)),
    }
}

fn lower_variant(state: &StateModel) -> VariantIr {
    VariantIr {
        docs: state.docs.clone(),
        name: state.variant.clone(),
        constructor: state.constructor.clone(),
        fields: state.fields.iter().map(lower_field).collect(),
    }
}

fn lower_field(field: &FieldModel) -> FieldIr {
    let default = field.default.as_ref().map(|default| match default {
        DefaultValue::Default => parse_quote!(::core::default::Default::default()),
        DefaultValue::Expr(expression) => expression.clone(),
    });

    FieldIr {
        name: field.name.clone(),
        ty: field.ty.clone(),
        default,
    }
}

fn lower_arm(
    variant: &Ident,
    fields: &[FieldModel],
    handler: &Ident,
    inputs: &[InputKind],
) -> ArmIr {
    ArmIr {
        pattern: lower_pattern(variant, fields, inputs),
        call: lower_call(handler, inputs),
    }
}

// MARK: Superstates

/// Return the type of the superstate enum, or `None` if the machine has no superstates.
fn lower_superstate_type(model: &Model) -> Option<SuperstateTypeIr> {
    if model.superstates.is_empty() {
        return None;
    }

    Some(SuperstateTypeIr {
        name: Ident::new("Superstate", Span::call_site()),
        has_lifetime: model
            .superstates
            .iter()
            .any(|superstate| !superstate.fields.is_empty()),
    })
}

fn lower_superstate(model: &Model, ty: SuperstateTypeIr, state: Ident) -> SuperstateIr {
    let takes = |input: &InputKind| {
        model
            .superstates
            .iter()
            .any(|superstate| superstate.inputs.contains(input))
    };

    SuperstateIr {
        variants: model
            .superstates
            .iter()
            .map(lower_superstate_variant)
            .collect(),
        shared_storage: model.shared_storage.clone(),
        generics: model.item.generics.clone(),
        event_type: model.event_type.clone(),
        context_type: model.context_type.clone(),
        exit_type: model.exit_type.clone(),
        state,
        shared_storage_parameter: parameter(SHARED_STORAGE, takes(&InputKind::Receiver)),
        event_parameter: parameter(EVENT, takes(&InputKind::Event)),
        context_parameter: parameter(CONTEXT, takes(&InputKind::Context)),
        arms: model
            .superstates
            .iter()
            .map(|superstate| {
                lower_arm(
                    &superstate.variant,
                    &superstate.fields,
                    &superstate.handler,
                    &superstate.inputs,
                )
            })
            .collect(),
        entry_actions: lower_superstate_actions(model, |superstate| {
            superstate.entry_action.as_ref()
        }),
        exit_actions: lower_superstate_actions(model, |superstate| superstate.exit_action.as_ref()),
        parents: lower_superstate_parents(model, ty.clone()),
        ty,
    }
}

fn lower_superstate_variant(superstate: &SuperstateModel) -> SuperstateVariantIr {
    SuperstateVariantIr {
        docs: superstate.docs.clone(),
        name: superstate.variant.clone(),
        constructor: superstate.constructor.clone(),
        initial: superstate.initial.clone(),
        fields: superstate.fields.iter().map(lower_field).collect(),
    }
}

// MARK: Parents

fn lower_state_parents(model: &Model, superstate: SuperstateTypeIr) -> ParentDispatchIr {
    ParentDispatchIr {
        superstate,
        arms: model
            .states
            .iter()
            .map(|state| {
                let parent = find_superstate(model, state.superstate.as_ref());

                lower_parent_arm(&state.variant, &state.fields, parent, |_| None)
            })
            .collect(),
    }
}

fn lower_superstate_parents(model: &Model, superstate: SuperstateTypeIr) -> ParentDispatchIr {
    ParentDispatchIr {
        superstate,
        arms: model
            .superstates
            .iter()
            .map(|child| {
                let parent = find_superstate(model, child.superstate.as_ref());

                lower_parent_arm(&child.variant, &child.fields, parent, |field| {
                    Some(reborrow(field))
                })
            })
            .collect(),
    }
}

fn find_superstate<'model>(
    model: &'model Model,
    link: Option<&Ident>,
) -> Option<&'model SuperstateModel> {
    let name = link?;

    model
        .superstates
        .iter()
        .find(|superstate| superstate.handler == *name)
}

fn lower_parent_arm(
    variant: &Ident,
    fields: &[FieldModel],
    parent: Option<&SuperstateModel>,
    value: impl Fn(&FieldModel) -> Option<Expr>,
) -> ParentArmIr {
    let parent_fields = parent.map_or(&[][..], |parent| parent.fields.as_slice());
    let bindings: Vec<Ident> = parent_fields
        .iter()
        .map(|field| field.name.clone())
        .collect();

    ParentArmIr {
        pattern: PatternIr {
            variant: variant.clone(),
            has_rest: bindings.len() < fields.len(),
            bindings,
        },
        parent: parent.map(|parent| ParentIr {
            variant: parent.variant.clone(),
            fields: parent_fields
                .iter()
                .map(|field| ParentFieldIr {
                    name: field.name.clone(),
                    value: value(field),
                })
                .collect(),
        }),
    }
}

/// Return the expression that reborrows the child's binding of the parent field `field`.
///
/// The binding is a reference to the child's own reference, so the parent's
/// borrow lasts as long as the borrow of the child. The mutability follows the
/// parent, which may borrow a field only for reading while its child borrows it
/// mutably.
fn reborrow(field: &FieldModel) -> Expr {
    let name = &field.name;

    match &field.ty {
        Type::Reference(reference) if reference.mutability.is_none() => {
            parse_quote!(&**#name)
        }
        _ => parse_quote!(&mut **#name),
    }
}

// MARK: Actions

fn lower_state_actions(
    model: &Model,
    attached: fn(&StateModel) -> Option<&Ident>,
) -> Option<ActionDispatchIr> {
    let nodes = model.states.iter().map(|state| ActionNode {
        variant: &state.variant,
        fields: &state.fields,
        action: attached(state),
    });

    lower_action_dispatch(&model.actions, nodes.collect())
}

fn lower_superstate_actions(
    model: &Model,
    attached: fn(&SuperstateModel) -> Option<&Ident>,
) -> Option<ActionDispatchIr> {
    let nodes = model.superstates.iter().map(|superstate| ActionNode {
        variant: &superstate.variant,
        fields: &superstate.fields,
        action: attached(superstate),
    });

    lower_action_dispatch(&model.actions, nodes.collect())
}

/// Lower the method that runs the action attached to each of `nodes`.
///
/// Returns `None` when no node has an action, which keeps the no-op default of
/// the trait.
fn lower_action_dispatch(
    actions: &[ActionModel],
    nodes: Vec<ActionNode<'_>>,
) -> Option<ActionDispatchIr> {
    if nodes.iter().all(|node| node.action.is_none()) {
        return None;
    }

    let resolve = |node: &ActionNode<'_>| {
        let name = node.action?;

        actions.iter().find(|action| action.name == *name)
    };

    let takes = |input: &InputKind| {
        nodes
            .iter()
            .filter_map(&resolve)
            .any(|action| action.inputs.contains(input))
    };

    let arms = nodes
        .iter()
        .map(|node| lower_action_arm(node, resolve(node)))
        .collect();

    Some(ActionDispatchIr {
        shared_storage_parameter: parameter(SHARED_STORAGE, takes(&InputKind::Receiver)),
        context_parameter: parameter(CONTEXT, takes(&InputKind::Context)),
        arms,
    })
}

fn lower_action_arm(node: &ActionNode<'_>, action: Option<&ActionModel>) -> ActionArmIr {
    match action {
        Some(action) => ActionArmIr {
            pattern: lower_pattern(node.variant, node.fields, &action.inputs),
            call: Some(lower_call(&action.name, &action.inputs)),
        },
        None => ActionArmIr {
            pattern: lower_pattern(node.variant, node.fields, &[]),
            call: None,
        },
    }
}

// MARK: Calls

fn lower_pattern(variant: &Ident, fields: &[FieldModel], inputs: &[InputKind]) -> PatternIr {
    let bindings: Vec<Ident> = inputs
        .iter()
        .filter_map(|input| match input {
            InputKind::LocalStorage { name, .. } => Some(name.clone()),
            InputKind::Receiver | InputKind::Event | InputKind::Context => None,
        })
        .collect();

    PatternIr {
        variant: variant.clone(),
        has_rest: bindings.len() < fields.len(),
        bindings,
    }
}

fn lower_call(function: &Ident, inputs: &[InputKind]) -> CallIr {
    CallIr {
        function: function.clone(),
        arguments: inputs.iter().map(argument).collect(),
    }
}

fn argument(input: &InputKind) -> Ident {
    match input {
        InputKind::Receiver => generated_ident(SHARED_STORAGE),
        InputKind::Event => generated_ident(EVENT),
        InputKind::Context => generated_ident(CONTEXT),
        InputKind::LocalStorage { name, .. } => name.clone(),
    }
}

/// Name a parameter of `call_handler`, underscored when no handler takes `input`.
fn handler_parameter(model: &Model, input: &InputKind, name: &str) -> Ident {
    let is_used = model
        .states
        .iter()
        .any(|state| state.inputs.contains(input));

    parameter(name, is_used)
}

fn parameter(name: &str, is_used: bool) -> Ident {
    if is_used {
        generated_ident(name)
    } else {
        generated_ident(&format!("_{name}"))
    }
}

/// Return an identifier for a parameter of a generated method.
///
/// Mixed-site hygiene keeps it apart from a local storage field of the same
/// name, which the match arms bind with the span of the user's code.
fn generated_ident(name: &str) -> Ident {
    Ident::new(name, Span::mixed_site())
}
