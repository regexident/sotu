//! Turn the lowered machine into tokens.

use proc_macro2::TokenStream;
use quote::quote;

use crate::lower::{ArmIr, CallIr, DispatchIr, FieldIr, Ir, MachineIr, PatternIr, StateEnumIr};

// MARK: Generation

/// Generate the impl block, the state enum and the trait implementations of a machine.
pub(crate) fn generate(ir: &Ir) -> TokenStream {
    let item = &ir.item;
    let state_enum = state_enum(&ir.state_enum);
    let into_state_machine = into_state_machine_impl(&ir.machine);
    let dispatch = dispatch_impl(&ir.dispatch);
    let submachine = submachine_impls(&ir.machine);

    quote! {
        #item

        #state_enum

        #into_state_machine

        #dispatch

        #submachine
    }
}

// MARK: State Enum

fn state_enum(state_enum: &StateEnumIr) -> TokenStream {
    let name = &state_enum.name;

    let variants = state_enum.variants.iter().map(|variant| {
        let docs = &variant.docs;
        let variant_name = &variant.name;
        let field_names = variant.fields.iter().map(|field| &field.name);
        let field_types = variant.fields.iter().map(|field| &field.ty);

        quote! {
            #(#docs)*
            #variant_name {
                #(#field_names: #field_types),*
            }
        }
    });

    let constructors = state_enum.variants.iter().map(|variant| {
        let constructor = &variant.constructor;
        let variant_name = &variant.name;

        let parameters = variant
            .fields
            .iter()
            .filter(|field| field.default.is_none())
            .map(|FieldIr { name, ty, .. }| quote!(#name: #ty));

        let initializers = variant.fields.iter().map(field_initializer);

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

fn field_initializer(field: &FieldIr) -> TokenStream {
    let name = &field.name;

    match &field.default {
        Some(default) => quote!(#name: #default),
        None => quote!(#name),
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
        initial,
    } = machine;

    quote! {
        type Event = #event_type;
        type Context<'ctx, 'data: 'ctx> = ();
        type State = #state;
        type Superstate<'sub> = ();
        type Exit = ::core::convert::Infallible;

        fn initial() -> #state {
            #initial
        }
    }
}

// MARK: Dispatch

fn dispatch_impl(dispatch: &DispatchIr) -> TokenStream {
    let DispatchIr {
        shared_storage,
        event_type,
        state,
        shared_storage_parameter,
        event_parameter,
        arms,
    } = dispatch;

    let arms = arms.iter().map(|arm| dispatch_arm(dispatch, arm));

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
        }
    }
}

fn dispatch_arm(dispatch: &DispatchIr, arm: &ArmIr) -> TokenStream {
    let pattern = pattern(dispatch, &arm.pattern);
    let call = call(dispatch, &arm.call);

    quote! {
        #pattern => #call,
    }
}

fn pattern(dispatch: &DispatchIr, pattern: &PatternIr) -> TokenStream {
    let state = &dispatch.state;

    let PatternIr {
        variant,
        bindings,
        has_rest,
    } = pattern;

    let rest = has_rest.then(|| quote!(..));

    quote! {
        #state::#variant { #(#bindings,)* #rest }
    }
}

fn call(dispatch: &DispatchIr, call: &CallIr) -> TokenStream {
    let shared_storage = &dispatch.shared_storage;
    let CallIr {
        function,
        arguments,
    } = call;

    quote! {
        #shared_storage::#function(#(#arguments),*)
    }
}
