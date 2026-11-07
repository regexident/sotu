//! Turn the lowered machine into tokens.

use proc_macro2::TokenStream;
use quote::quote;

use crate::lower::{ArmIr, DispatchIr, Ir, MachineIr, StateEnumIr};

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

        quote! {
            #(#docs)*
            #variant_name {}
        }
    });

    let constructors = state_enum.variants.iter().map(|variant| {
        let constructor = &variant.constructor;
        let variant_name = &variant.name;

        quote! {
            pub fn #constructor() -> Self {
                Self::#variant_name {}
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
    let shared_storage = &dispatch.shared_storage;
    let state = &dispatch.state;

    let ArmIr {
        variant,
        handler,
        arguments,
    } = arm;

    quote! {
        #state::#variant {} => #shared_storage::#handler(#(#arguments),*),
    }
}
