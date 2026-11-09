//! Turn the lowered machine into tokens.

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{Attribute, Expr, Path, PathArguments, Token, Type, parse_quote};

use crate::analyze::HooksModel;
use crate::lower::{
    ActionArmIr, ActionDispatchIr, ArmIr, CallIr, DispatchIr, FieldIr, Ir, MachineIr, ParentArmIr,
    ParentDispatchIr, ParentFieldIr, ParentIr, PatternIr, StateEnumIr, SuperstateIr,
    SuperstateTypeIr,
};

// MARK: Generation

/// Generate the impl block, the node enums and the trait implementations of a machine.
pub(crate) fn generate(ir: &Ir) -> TokenStream {
    let item = &ir.item;
    let state_enum = ir.state_enum.as_ref().map(state_enum);
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
    let visibility = &state_enum.visibility;

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
            #visibility fn #constructor(#(#parameters),*) -> Self {
                Self::#variant_name {
                    #(#initializers),*
                }
            }
        }
    });

    quote! {
        #visibility enum #name {
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

/// Return the tokens that name the superstate enum, using `lifetime` if it takes one.
fn superstate_type(ty: &SuperstateTypeIr, lifetime: &TokenStream) -> TokenStream {
    let name = &ty.name;

    if ty.has_lifetime {
        quote!(#name<#lifetime>)
    } else {
        quote!(#name)
    }
}

fn superstate_enum(superstate: &SuperstateIr) -> TokenStream {
    let declared = superstate_type(&superstate.ty, &quote!('sub));
    let elided = superstate_type(&superstate.ty, &quote!('_));
    let state = &superstate.state;
    let visibility = &superstate.visibility;

    let variants = superstate
        .variants
        .iter()
        .map(|variant| enum_variant(&variant.docs, &variant.name, &variant.fields));

    let constructors: Vec<TokenStream> = superstate
        .variants
        .iter()
        .filter_map(|variant| {
            let constructor = &variant.constructor;
            let initial = variant.initial.as_ref()?;

            Some(quote! {
                #visibility fn #constructor() -> #state {
                    #initial
                }
            })
        })
        .collect();

    let constructor_impl = (!constructors.is_empty()).then(|| {
        quote! {
            impl #elided {
                #(#constructors)*
            }
        }
    });

    quote! {
        #visibility enum #declared {
            #(#variants,)*
        }

        #constructor_impl
    }
}

// MARK: Machine Traits

fn into_state_machine_impl(machine: &MachineIr) -> TokenStream {
    let shared_storage = &machine.shared_storage;
    let (impl_generics, _, where_clause) = machine.generics.split_for_impl();
    let items = machine_items(machine);
    let hooks = hook_methods(machine);

    quote! {
        impl #impl_generics ::sotu::blocking::IntoStateMachine for #shared_storage #where_clause {
            #items

            #hooks
        }
    }
}

/// Generate the hook methods whose key is present, each delegating to the arity
/// adapter that accepts the hook with or without a context parameter.
///
/// An absent key generates no method, which keeps the no-op default of the trait.
fn hook_methods(machine: &MachineIr) -> TokenStream {
    let HooksModel {
        before_dispatch,
        after_dispatch,
        before_transition,
        after_transition,
    } = &machine.hooks;

    let before_dispatch = before_dispatch
        .as_ref()
        .map(|path| dispatch_hook(machine, &format_ident!("before_dispatch"), path));

    let after_dispatch = after_dispatch
        .as_ref()
        .map(|path| dispatch_hook(machine, &format_ident!("after_dispatch"), path));

    let before_transition = before_transition
        .as_ref()
        .map(|path| transition_hook(machine, &format_ident!("before_transition"), path));

    let after_transition = after_transition
        .as_ref()
        .map(|path| transition_hook(machine, &format_ident!("after_transition"), path));

    quote! {
        #before_dispatch

        #after_dispatch

        #before_transition

        #after_transition
    }
}

fn dispatch_hook(machine: &MachineIr, method: &Ident, path: &Path) -> TokenStream {
    let MachineIr {
        event_type,
        context_type,
        state,
        superstate,
        ..
    } = machine;

    let superstate = match superstate {
        Some(ty) => superstate_type(ty, &quote!('_)),
        None => quote!(()),
    };

    quote! {
        fn #method<'ctx, 'data: 'ctx>(
            &mut self,
            node: ::sotu::StateOrSuperstate<'_, #state, #superstate>,
            event: &#event_type,
            context: &mut #context_type,
        ) {
            ::sotu::blocking::DispatchHook::call_dispatch_hook(&(#path), self, node, event, context)
        }
    }
}

fn transition_hook(machine: &MachineIr, method: &Ident, path: &Path) -> TokenStream {
    let MachineIr {
        context_type,
        state,
        ..
    } = machine;

    quote! {
        fn #method<'ctx, 'data: 'ctx>(
            &mut self,
            source: &#state,
            target: &#state,
            context: &mut #context_type,
        ) {
            ::sotu::blocking::TransitionHook::call_transition_hook(
                &(#path),
                self,
                source,
                target,
                context,
            )
        }
    }
}

fn submachine_impls(machine: &MachineIr) -> TokenStream {
    let shared_storage = &machine.shared_storage;
    let (impl_generics, _, where_clause) = machine.generics.split_for_impl();
    let items = machine_items(machine);

    quote! {
        impl #impl_generics ::sotu::private::Submachine for #shared_storage #where_clause {
            #items
        }

        impl #impl_generics ::sotu::private::BlockingSubmachine for #shared_storage
        #where_clause
        {}
    }
}

/// Generate the associated types and `initial` that both machine traits declare.
fn machine_items(machine: &MachineIr) -> TokenStream {
    let MachineIr {
        shared_storage: _,
        generics: _,
        event_type,
        context_type,
        exit_type,
        state,
        superstate,
        initial,
        hooks: _,
    } = machine;

    let superstate = match superstate {
        Some(ty) => superstate_type(ty, &quote!('sub)),
        None => quote!(()),
    };

    quote! {
        type Event = #event_type;
        type Context<'ctx, 'data: 'ctx> = #context_type;
        type State = #state;
        type Superstate<'sub> = #superstate;
        type Exit = #exit_type;

        fn initial() -> #state {
            #initial
        }
    }
}

// MARK: State Dispatch

fn dispatch_impl(dispatch: &DispatchIr) -> TokenStream {
    let DispatchIr {
        shared_storage,
        generics,
        event_type,
        context_type,
        exit_type,
        state,
        shared_storage_parameter,
        event_parameter,
        context_parameter,
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
            context_type,
            &format_ident!("call_entry_action"),
            actions,
        )
    });

    let exit_action = exit_actions.as_ref().map(|actions| {
        action_method(
            state,
            shared_storage,
            context_type,
            &format_ident!("call_exit_action"),
            actions,
        )
    });

    let parents = parents
        .as_ref()
        .map(|parents| parent_method(state, parents));

    let (impl_generics, _, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics ::sotu::blocking::State<#shared_storage> for #state #where_clause {
            fn call_handler<'ctx, 'data: 'ctx>(
                &mut self,
                #shared_storage_parameter: &mut #shared_storage,
                #event_parameter: &#event_type,
                #context_parameter: &mut #context_type,
            ) -> ::sotu::Outcome<Self, #exit_type> {
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
        visibility: _,
        variants,
        shared_storage,
        generics,
        event_type,
        context_type,
        exit_type,
        state,
        shared_storage_parameter,
        event_parameter,
        context_parameter,
        arms,
        entry_actions,
        exit_actions,
        parents,
    } = superstate;

    let name = &ty.name;
    let declared = superstate_type(ty, &quote!('sub));
    let elided = superstate_type(ty, &quote!('_));
    let mut generics = generics.clone();

    if ty.has_lifetime {
        generics.params.insert(0, parse_quote!('sub));
    }

    let (impl_generics, _, where_clause) = generics.split_for_impl();

    let arms = arms
        .iter()
        .map(|arm| dispatch_arm(name, shared_storage, arm));

    let entry_action = entry_actions.as_ref().map(|actions| {
        action_method(
            name,
            shared_storage,
            context_type,
            &format_ident!("call_entry_action"),
            actions,
        )
    });

    let exit_action = exit_actions.as_ref().map(|actions| {
        action_method(
            name,
            shared_storage,
            context_type,
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
        impl #impl_generics ::sotu::blocking::Superstate<#shared_storage> for #declared
        #where_clause
        {
            fn call_handler<'ctx, 'data: 'ctx>(
                &mut self,
                #shared_storage_parameter: &mut #shared_storage,
                #event_parameter: &#event_type,
                #context_parameter: &mut #context_type,
            ) -> ::sotu::Outcome<#state, #exit_type> {
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
    context_type: &Type,
    method: &Ident,
    actions: &ActionDispatchIr,
) -> TokenStream {
    let shared_storage_parameter = &actions.shared_storage_parameter;
    let context_parameter = &actions.context_parameter;

    let arms = actions
        .arms
        .iter()
        .map(|arm| action_arm(enum_name, shared_storage, arm));

    quote! {
        fn #method<'ctx, 'data: 'ctx>(
            &mut self,
            #shared_storage_parameter: &mut #shared_storage,
            #context_parameter: &mut #context_type,
        ) {
            match self {
                #(#arms)*
            }
        }
    }
}

fn parent_method(enum_name: &Ident, parents: &ParentDispatchIr) -> TokenStream {
    let elided = superstate_type(&parents.superstate, &quote!('_));

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

    let path = expression_path(shared_storage);

    quote! {
        #path::#function(#(#arguments),*)
    }
}

/// Write `ty` as the path that qualifies a call to one of its associated functions.
///
/// Generic arguments in expression position need the turbofish, as in
/// `Tagger::<'a>::tagging`. A type that is not a path is qualified as `<Type>`.
fn expression_path(ty: &Type) -> TokenStream {
    let Type::Path(path) = ty else {
        return quote!(<#ty>);
    };

    let mut path = path.clone();

    for segment in &mut path.path.segments {
        if let PathArguments::AngleBracketed(arguments) = &mut segment.arguments {
            arguments.colon2_token = Some(<Token![::]>::default());
        }
    }

    quote!(#path)
}
