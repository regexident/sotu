//! Resolve every name, type and call the generated code needs.

use proc_macro2::Span;
use syn::{Attribute, Expr, Ident, ItemImpl, Type};

use crate::analyze::{InputKind, Model, StateModel};

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
}

/// A match arm that calls one handler.
pub(crate) struct ArmIr {
    /// The variant the arm matches.
    pub(crate) variant: Ident,
    /// The handler the arm calls.
    pub(crate) handler: Ident,
    /// The arguments of the call, in the order the handler declares its inputs.
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
            shared_storage_parameter: parameter(model, InputKind::Receiver, SHARED_STORAGE),
            event_parameter: parameter(model, InputKind::Event, EVENT),
            arms: model.states.iter().map(lower_arm).collect(),
        },
    }
}

fn lower_variant(state: &StateModel) -> VariantIr {
    VariantIr {
        docs: state.docs.clone(),
        name: state.variant.clone(),
        constructor: state.constructor.clone(),
    }
}

fn lower_arm(state: &StateModel) -> ArmIr {
    ArmIr {
        variant: state.variant.clone(),
        handler: state.handler.clone(),
        arguments: state.inputs.iter().copied().map(argument).collect(),
    }
}

fn argument(input: InputKind) -> Ident {
    let name = match input {
        InputKind::Receiver => SHARED_STORAGE,
        InputKind::Event => EVENT,
    };

    Ident::new(name, Span::call_site())
}

/// Name a parameter of `call_handler`, underscored when no handler takes `input`.
fn parameter(model: &Model, input: InputKind, name: &str) -> Ident {
    let is_used = model
        .states
        .iter()
        .any(|state| state.inputs.contains(&input));

    if is_used {
        Ident::new(name, Span::call_site())
    } else {
        Ident::new(&format!("_{name}"), Span::call_site())
    }
}
