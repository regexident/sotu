//! Turn the lowered machine into tokens.

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{Attribute, Expr, Type};

use crate::lower::{
    ActionArmIr, ActionDispatchIr, ArmIr, CallIr, DispatchIr, FieldIr, Ir, MachineIr, ParentArmIr,
    ParentDispatchIr, ParentFieldIr, ParentIr, PatternIr, StateEnumIr, SuperstateIr,
    SuperstateTypeIr,
};

// MARK: Generation

/// Generate the impl block, the node enums and the trait implementations of a machine.
pub(crate) fn generate(ir: &Ir) -> TokenStream {
    let item = &ir.item;
    let state_enum = state_enum(&ir.state_enum);
    let superstate_enum = ir.superstate.as_ref().map(superstate_enum);
    let into_state_machine = into_state_machine_impl(&ir.machine);
    let dispatch = dispatch_impl(&ir.dispatch);
    let superstate_dispatch = ir.superstate.as_ref().map(superstate_dispatch_impl);
    let submachine = submachine_impls(&ir.machine);

    quote! {
        #item

        #state_enum

        #superstate_enum

        #into_state_machine

        #dispatch

        #superstate_dispatch

        #submachine
    }
}

// MARK: State Enum

fn state_enum(state_enum: &StateEnumIr) -> TokenStream {
    let name = &state_enum.name;

    let variants = state_enum
        .variants
        .iter()
        .map(|variant| enum_variant(&variant.docs, &variant.name, &variant.fields));

    let constructors = state_enum.variants.iter().map(|variant| {
        let constructor = &variant.constructor;
        let variant_name = &variant.name;

        let parameters = variant
            .fields
            .iter()
            .filter(|field| field.default.is_none())
            .map(|FieldIr { name, ty, .. }| quote!(#name: #ty));

        let initializers = variant
            .fields
            .iter()
            .map(|field| field_initializer(&field.name, field.default.as_ref()));

        quote! {
            pub fn #constructor(#(#parameters),*) -> Self {
                Self::#variant_name {
                    #(#initializers),*
                }
            }
        }
    });

    quote! {
        pub enum #name {
            #(#variants,)*
        }

        impl #name {
            #(#constructors)*
        }
    }
}

fn enum_variant(docs: &[Attribute], name: &Ident, fields: &[FieldIr]) -> TokenStream {
    let field_names = fields.iter().map(|field| &field.name);
    let field_types = fields.iter().map(|field| &field.ty);

    quote! {
        #(#docs)*
        #name {
            #(#field_names: #field_types),*
        }
    }
}

/// Generate `name` in shorthand, or `name: value` when the field has its own value.
fn field_initializer(name: &Ident, value: Option<&Expr>) -> TokenStream {
    match value {
        Some(value) => quote!(#name: #value),
        None => quote!(#name),
    }
}

// MARK: Superstate Enum

/// Return the tokens that name the superstate enum.
fn superstate_type(ty: &SuperstateTypeIr) -> TokenStream {
    let name = &ty.name;

    quote!(#name)
}

fn superstate_enum(superstate: &SuperstateIr) -> TokenStream {
    let declared = superstate_type(&superstate.ty);

    let variants = superstate
        .variants
        .iter()
        .map(|variant| enum_variant(&variant.docs, &variant.name, &[]));

    quote! {
        pub enum #declared {
            #(#variants,)*
        }
    }
}

// MARK: Machine Traits

fn into_state_machine_impl(machine: &MachineIr) -> TokenStream {
    let shared_storage = &machine.shared_storage;
    let items = machine_items(machine);

    quote! {
        impl ::sotu::blocking::IntoStateMachine for #shared_storage {
            #items
        }
    }
}

fn submachine_impls(machine: &MachineIr) -> TokenStream {
    let shared_storage = &machine.shared_storage;
    let items = machine_items(machine);

    quote! {
        impl ::sotu::private::Submachine for #shared_storage {
            #items
        }

        impl ::sotu::private::BlockingSubmachine for #shared_storage {}
    }
}

/// Generate the associated types and `initial` that both machine traits declare.
fn machine_items(machine: &MachineIr) -> TokenStream {
    let MachineIr {
        shared_storage: _,
        event_type,
        state,
        superstate,
        initial,
    } = machine;

    let superstate = match superstate {
        Some(ty) => superstate_type(ty),
        None => quote!(()),
    };

    quote! {
        type Event = #event_type;
        type Context<'ctx, 'data: 'ctx> = ();
        type State = #state;
        type Superstate<'sub> = #superstate;
        type Exit = ::core::convert::Infallible;

        fn initial() -> #state {
            #initial
        }
    }
}

// MARK: State Dispatch

fn dispatch_impl(dispatch: &DispatchIr) -> TokenStream {
    let DispatchIr {
        shared_storage,
        event_type,
        state,
        shared_storage_parameter,
        event_parameter,
        arms,
        entry_actions,
        exit_actions,
        parents,
    } = dispatch;

    let arms = arms
        .iter()
        .map(|arm| dispatch_arm(state, shared_storage, arm));

    let entry_action = entry_actions.as_ref().map(|actions| {
        action_method(
            state,
            shared_storage,
            &format_ident!("call_entry_action"),
            actions,
        )
    });

    let exit_action = exit_actions.as_ref().map(|actions| {
        action_method(
            state,
            shared_storage,
            &format_ident!("call_exit_action"),
            actions,
        )
    });

    let parents = parents
        .as_ref()
        .map(|parents| parent_method(state, parents));

    quote! {
        impl ::sotu::blocking::State<#shared_storage> for #state {
            fn call_handler<'ctx, 'data: 'ctx>(
                &mut self,
                #shared_storage_parameter: &mut #shared_storage,
                #event_parameter: &#event_type,
                _context: &mut (),
            ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
                match self {
                    #(#arms)*
                }
            }

            #entry_action

            #exit_action

            #parents
        }
    }
}

// MARK: Superstate Dispatch

fn superstate_dispatch_impl(superstate: &SuperstateIr) -> TokenStream {
    let SuperstateIr {
        ty,
        variants,
        shared_storage,
        event_type,
        state,
        shared_storage_parameter,
        event_parameter,
        arms,
        entry_actions,
        exit_actions,
        parents,
    } = superstate;

    let name = &ty.name;
    let declared = superstate_type(ty);
    let elided = superstate_type(ty);

    let arms = arms
        .iter()
        .map(|arm| dispatch_arm(name, shared_storage, arm));

    let entry_action = entry_actions.as_ref().map(|actions| {
        action_method(
            name,
            shared_storage,
            &format_ident!("call_entry_action"),
            actions,
        )
    });

    let exit_action = exit_actions.as_ref().map(|actions| {
        action_method(
            name,
            shared_storage,
            &format_ident!("call_exit_action"),
            actions,
        )
    });

    let parents = parent_method(name, parents);

    let same_variant_arms = variants.iter().map(|variant| {
        let variant_name = &variant.name;

        quote! {
            (#name::#variant_name { .. }, #name::#variant_name { .. }) => true,
        }
    });

    // With a single variant the pairs above are exhaustive, and a wildcard arm
    // would be reported as unreachable in the user's crate.
    let different_variants = (variants.len() > 1).then(|| quote!(_ => false,));

    quote! {
        impl ::sotu::blocking::Superstate<#shared_storage> for #declared {
            fn call_handler<'ctx, 'data: 'ctx>(
                &mut self,
                #shared_storage_parameter: &mut #shared_storage,
                #event_parameter: &#event_type,
                _context: &mut (),
            ) -> ::sotu::Outcome<#state, ::core::convert::Infallible> {
                match self {
                    #(#arms)*
                }
            }

            #entry_action

            #exit_action

            #parents

            fn same_state(&self, other: &#elided) -> bool {
                match (self, other) {
                    #(#same_variant_arms)*
                    #different_variants
                }
            }
        }
    }
}

// MARK: Shared Dispatch

fn action_method(
    enum_name: &Ident,
    shared_storage: &Type,
    method: &Ident,
    actions: &ActionDispatchIr,
) -> TokenStream {
    let shared_storage_parameter = &actions.shared_storage_parameter;

    let arms = actions
        .arms
        .iter()
        .map(|arm| action_arm(enum_name, shared_storage, arm));

    quote! {
        fn #method<'ctx, 'data: 'ctx>(
            &mut self,
            #shared_storage_parameter: &mut #shared_storage,
            _context: &mut (),
        ) {
            match self {
                #(#arms)*
            }
        }
    }
}

fn parent_method(enum_name: &Ident, parents: &ParentDispatchIr) -> TokenStream {
    let elided = superstate_type(&parents.superstate);

    let arms = parents
        .arms
        .iter()
        .map(|arm| parent_arm(enum_name, &parents.superstate.name, arm));

    quote! {
        fn superstate(&mut self) -> ::core::option::Option<#elided> {
            match self {
                #(#arms)*
            }
        }
    }
}

fn parent_arm(enum_name: &Ident, superstate: &Ident, arm: &ParentArmIr) -> TokenStream {
    let pattern = pattern(enum_name, &arm.pattern);

    match &arm.parent {
        Some(ParentIr { variant, fields }) => {
            let fields = fields
                .iter()
                .map(|ParentFieldIr { name, value }| field_initializer(name, value.as_ref()));

            quote! {
                #pattern => ::core::option::Option::Some(#superstate::#variant {
                    #(#fields),*
                }),
            }
        }
        None => quote! {
            #pattern => ::core::option::Option::None,
        },
    }
}

fn dispatch_arm(enum_name: &Ident, shared_storage: &Type, arm: &ArmIr) -> TokenStream {
    let pattern = pattern(enum_name, &arm.pattern);
    let call = call(shared_storage, &arm.call);

    quote! {
        #pattern => #call,
    }
}

fn action_arm(enum_name: &Ident, shared_storage: &Type, arm: &ActionArmIr) -> TokenStream {
    let pattern = pattern(enum_name, &arm.pattern);

    match &arm.call {
        Some(action) => {
            let call = call(shared_storage, action);

            quote! {
                #pattern => #call,
            }
        }
        None => quote! {
            #pattern => {}
        },
    }
}

fn pattern(enum_name: &Ident, pattern: &PatternIr) -> TokenStream {
    let PatternIr {
        variant,
        bindings,
        has_rest,
    } = pattern;

    let rest = has_rest.then(|| quote!(..));

    quote! {
        #enum_name::#variant { #(#bindings,)* #rest }
    }
}

fn call(shared_storage: &Type, call: &CallIr) -> TokenStream {
    let CallIr {
        function,
        arguments,
    } = call;

    quote! {
        #shared_storage::#function(#(#arguments),*)
    }
}
