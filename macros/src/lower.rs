//! Resolve every name, type and call the generated code needs.

use proc_macro2::Span;
use syn::{Attribute, Expr, Ident, ItemImpl, Type, parse_quote};

use crate::analyze::{ActionModel, DefaultValue, FieldModel, InputKind, Model, StateModel};

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
    /// The event type of the machine.
    pub(crate) event_type: Type,
    /// The name of the state enum.
    pub(crate) state: Ident,
    /// The expression that builds the initial state.
    pub(crate) initial: Expr,
}

/// The implementation of the blocking `State` trait.
pub(crate) struct DispatchIr {
    /// The type the handlers belong to.
    pub(crate) shared_storage: Type,
    /// The event type of the machine.
    pub(crate) event_type: Type,
    /// The name of the state enum.
    pub(crate) state: Ident,
    /// The name of the shared storage parameter, underscored when unused.
    pub(crate) shared_storage_parameter: Ident,
    /// The name of the event parameter, underscored when unused.
    pub(crate) event_parameter: Ident,
    /// One match arm per variant, in declaration order.
    pub(crate) arms: Vec<ArmIr>,
    /// The body of `call_entry_action`, if any state has an entry action.
    pub(crate) entry_actions: Option<ActionDispatchIr>,
    /// The body of `call_exit_action`, if any state has an exit action.
    pub(crate) exit_actions: Option<ActionDispatchIr>,
}

/// A match arm that calls one handler.
pub(crate) struct ArmIr {
    /// The pattern the arm matches.
    pub(crate) pattern: PatternIr,
    /// The call to the handler.
    pub(crate) call: CallIr,
}

/// The method that runs the entry or exit action of the active state.
pub(crate) struct ActionDispatchIr {
    /// The name of the shared storage parameter, underscored when unused.
    pub(crate) shared_storage_parameter: Ident,
    /// One match arm per variant, in declaration order.
    pub(crate) arms: Vec<ActionArmIr>,
}

/// A match arm that runs the action of one state, if it has one.
pub(crate) struct ActionArmIr {
    /// The pattern the arm matches.
    pub(crate) pattern: PatternIr,
    /// The call to the action.
    pub(crate) call: Option<CallIr>,
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

// MARK: Lowering

const SHARED_STORAGE: &str = "shared_storage";
const EVENT: &str = "event";

/// Resolve `model` into the [`Ir`] that code generation reads.
pub(crate) fn lower(model: &Model) -> Ir {
    let state = Ident::new("State", Span::call_site());

    Ir {
        item: model.item.clone(),
        state_enum: StateEnumIr {
            name: state.clone(),
            variants: model.states.iter().map(lower_variant).collect(),
        },
        machine: MachineIr {
            shared_storage: model.shared_storage.clone(),
            event_type: model.event_type.clone(),
            state: state.clone(),
            initial: model.initial.clone(),
        },
        dispatch: DispatchIr {
            shared_storage: model.shared_storage.clone(),
            event_type: model.event_type.clone(),
            state,
            shared_storage_parameter: handler_parameter(
                model,
                &InputKind::Receiver,
                SHARED_STORAGE,
            ),
            event_parameter: handler_parameter(model, &InputKind::Event, EVENT),
            arms: model.states.iter().map(lower_arm).collect(),
            entry_actions: lower_action_dispatch(model, |state| state.entry_action.as_ref()),
            exit_actions: lower_action_dispatch(model, |state| state.exit_action.as_ref()),
        },
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

fn lower_arm(state: &StateModel) -> ArmIr {
    ArmIr {
        pattern: lower_pattern(state, &state.inputs),
        call: lower_call(&state.handler, &state.inputs),
    }
}

/// Lower the method that runs the action `attached` selects from each state.
///
/// Returns `None` when no state has such an action, which keeps the no-op
/// default of the `State` trait.
fn lower_action_dispatch(
    model: &Model,
    attached: fn(&StateModel) -> Option<&Ident>,
) -> Option<ActionDispatchIr> {
    let resolve = |state: &StateModel| -> Option<&ActionModel> {
        let name = attached(state)?;

        model.actions.iter().find(|action| action.name == *name)
    };

    if !model.states.iter().any(|state| attached(state).is_some()) {
        return None;
    }

    let takes_receiver = model
        .states
        .iter()
        .filter_map(resolve)
        .any(|action| action.inputs.contains(&InputKind::Receiver));

    let arms = model
        .states
        .iter()
        .map(|state| lower_action_arm(state, resolve(state)))
        .collect();

    Some(ActionDispatchIr {
        shared_storage_parameter: parameter(SHARED_STORAGE, takes_receiver),
        arms,
    })
}

fn lower_action_arm(state: &StateModel, action: Option<&ActionModel>) -> ActionArmIr {
    match action {
        Some(action) => ActionArmIr {
            pattern: lower_pattern(state, &action.inputs),
            call: Some(lower_call(&action.name, &action.inputs)),
        },
        None => ActionArmIr {
            pattern: lower_pattern(state, &[]),
            call: None,
        },
    }
}

fn lower_pattern(state: &StateModel, inputs: &[InputKind]) -> PatternIr {
    let bindings: Vec<Ident> = inputs
        .iter()
        .filter_map(|input| match input {
            InputKind::LocalStorage { name, .. } => Some(name.clone()),
            InputKind::Receiver | InputKind::Event => None,
        })
        .collect();

    PatternIr {
        variant: state.variant.clone(),
        has_rest: bindings.len() < state.fields.len(),
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
