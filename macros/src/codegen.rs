//! Turn the lowered machine into tokens.

use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    Attribute, Expr, GenericParam, Generics, Path, PathArguments, Token, Type, parse_quote,
    parse_quote_spanned,
};

use crate::analyze::HooksModel;
use crate::lower::{
    ActionArmIr, ActionDispatchIr, ArmIr, CallIr, DispatchIr, FieldIr, Ir, MachineIr, MountIr,
    NodeTypeIr, ParentArmIr, ParentDispatchIr, ParentFieldIr, ParentIr, PatternIr, StateEnumIr,
    SuperstateIr, generated_ident,
};

// MARK: Generation

/// Generate the impl block, the node enums and the trait implementations of a machine.
pub(crate) fn generate(ir: &Ir) -> TokenStream {
    let item = &ir.item;
    let mounts = &ir.mounts;

    let state_enum = ir
        .state_enum
        .as_ref()
        .map(|state_enum_ir| state_enum(state_enum_ir, mounts));

    let superstate_enum = ir
        .superstate
        .as_ref()
        .map(|superstate| superstate_enum(superstate, mounts));

    let into_state_machine = into_state_machine_impl(&ir.machine);
    let dispatch = dispatch_impl(&ir.dispatch, mounts);

    let superstate_dispatch = ir
        .superstate
        .as_ref()
        .map(|superstate| superstate_dispatch_impl(superstate, mounts));

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

fn state_enum(state_enum: &StateEnumIr, mounts: &[MountIr]) -> TokenStream {
    let name = &state_enum.ty.name;
    let generics = &state_enum.ty.generics;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    let visibility = &state_enum.visibility;
    let derives = derive_attribute(&state_enum.derives);

    let variants = state_enum
        .variants
        .iter()
        .map(|variant| enum_variant(&variant.docs, &variant.name, &variant.fields))
        .chain(
            mounts
                .iter()
                .map(|mount| mount_variant(mount, &mount.state_type)),
        );

    let mount_constructors = mounts.iter().map(|mount| {
        let MountIr {
            variant,
            constructor,
            state_type,
            ..
        } = mount;

        quote! {
            #visibility fn #constructor(state: #state_type) -> Self {
                Self::#variant(state)
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
        #derives
        #visibility enum #name #generics #where_clause {
            #(#variants,)*
        }

        impl #impl_generics #name #type_generics #where_clause {
            #(#constructors)*

            #(#mount_constructors)*
        }
    }
}

/// Generate the tuple variant of `mount` that holds a value of type `ty`.
fn mount_variant(mount: &MountIr, ty: &Type) -> TokenStream {
    let MountIr { docs, variant, .. } = mount;

    quote! {
        #(#docs)*
        #variant(#ty)
    }
}

/// Return the tokens that name the state enum with its parameters, as in `State<T>`.
fn state_type(ty: &NodeTypeIr) -> TokenStream {
    let name = &ty.name;
    let (_, type_generics, _) = ty.generics.split_for_impl();

    quote!(#name #type_generics)
}

/// Generate the `#[derive(...)]` attribute of a node enum, or nothing without `derives`.
fn derive_attribute(derives: &[Path]) -> Option<TokenStream> {
    (!derives.is_empty()).then(|| quote!(#[derive(#(#derives),*)]))
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

/// Return whether the superstate enum declares `'sub`, which is then its first parameter.
fn has_lifetime(ty: &NodeTypeIr) -> bool {
    matches!(
        ty.generics.params.first(),
        Some(GenericParam::Lifetime(parameter)) if parameter.lifetime.ident == "sub"
    )
}

/// Return the tokens that name the superstate enum with its parameters, using
/// `lifetime` in place of `'sub`, as in `Superstate<'_, T>`.
fn superstate_type(ty: &NodeTypeIr, lifetime: &TokenStream) -> TokenStream {
    let name = &ty.name;

    if ty.generics.params.is_empty() {
        return quote!(#name);
    }

    let arguments = ty.generics.params.iter().map(|parameter| match parameter {
        GenericParam::Lifetime(parameter) if parameter.lifetime.ident == "sub" => lifetime.clone(),
        GenericParam::Lifetime(parameter) => parameter.lifetime.to_token_stream(),
        GenericParam::Type(parameter) => parameter.ident.to_token_stream(),
        GenericParam::Const(parameter) => parameter.ident.to_token_stream(),
    });

    quote!(#name<#(#arguments),*>)
}

fn superstate_enum(superstate: &SuperstateIr, mounts: &[MountIr]) -> TokenStream {
    let name = &superstate.ty.name;
    let generics = &superstate.ty.generics;
    let where_clause = &generics.where_clause;
    let elided = superstate_type(&superstate.ty, &quote!('_));
    let state = state_type(&superstate.state);
    let constructor_generics = &superstate.constructor_generics;
    let constructor_where_clause = &constructor_generics.where_clause;
    let visibility = &superstate.visibility;
    let derives = derive_attribute(&superstate.derives);

    let variants = superstate
        .variants
        .iter()
        .map(|variant| enum_variant(&variant.docs, &variant.name, &variant.fields))
        .chain(
            mounts
                .iter()
                .map(|mount| mount_variant(mount, &mount.superstate_type)),
        );

    let initial_states = superstate.variants.iter().filter_map(|variant| {
        let initial = variant.initial.as_ref()?;

        Some((&variant.constructor, quote!(#initial)))
    });

    // A mount starts in the initial state of the child.
    let state_name = &superstate.state.name;

    let mount_states = mounts.iter().map(|mount| {
        let MountIr {
            variant,
            constructor,
            machine,
            ..
        } = mount;

        let initial = quote! {
            #state_name::#variant(<#machine as ::sotu::private::Submachine>::initial())
        };

        (constructor, initial)
    });

    let constructors: Vec<TokenStream> = initial_states
        .chain(mount_states)
        .map(|(constructor, initial)| {
            quote! {
                #visibility fn #constructor #constructor_generics() -> #state
                #constructor_where_clause
                {
                    #initial
                }
            }
        })
        .collect();

    let constructor_impl = (!constructors.is_empty()).then(|| {
        // The impl names the enum with `'_`, so it declares every parameter but `'sub`.
        let mut impl_generics = generics.clone();

        if has_lifetime(&superstate.ty) {
            impl_generics.params = impl_generics.params.into_iter().skip(1).collect();
        }

        let (impl_generics, _, _) = impl_generics.split_for_impl();

        quote! {
            impl #impl_generics #elided #where_clause {
                #(#constructors)*
            }
        }
    });

    quote! {
        #derives
        #visibility enum #name #generics #where_clause {
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

    let state = state_type(state);

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

    let state = state_type(state);

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

    // Without `exit = Type` the exit type is `Infallible`, so the machine can be
    // mounted without an exit handler.
    let exitless = (!machine.declares_exit).then(|| {
        quote! {
            impl #impl_generics ::sotu::private::ExitlessSubmachine for #shared_storage
            #where_clause
            {}
        }
    });

    quote! {
        impl #impl_generics ::sotu::private::Submachine for #shared_storage #where_clause {
            #items
        }

        impl #impl_generics ::sotu::private::BlockingSubmachine for #shared_storage
        #where_clause
        {}

        #exitless
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
        declares_exit: _,
        state,
        superstate,
        initial,
        hooks: _,
    } = machine;

    let state = state_type(state);

    let superstate = match superstate {
        Some(ty) => superstate_type(ty, &quote!('sub)),
        None => quote!(()),
    };

    quote! {
        type Event = #event_type;
        type Context<'ctx, 'data: 'ctx> = #context_type;
        type State = #state;
        type Superstate<'sub> = #superstate where Self: 'sub;
        type Exit = #exit_type;

        fn initial() -> #state {
            #initial
        }
    }
}

// MARK: State Dispatch

fn dispatch_impl(dispatch: &DispatchIr, mounts: &[MountIr]) -> TokenStream {
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

    let name = &state.name;
    let state = state_type(state);

    // Events reach the child's states only through the mount node, so a mount
    // variant defers every event to its superstates.
    let mount_arms = mounts.iter().map(|mount| {
        let variant = &mount.variant;

        quote! {
            #name::#variant(_) => ::sotu::Outcome::Super,
        }
    });

    let arms = arms
        .iter()
        .map(|arm| dispatch_arm(name, shared_storage, arm))
        .chain(mount_arms);

    let leaves: Vec<&Ident> = dispatch
        .arms
        .iter()
        .map(|arm| &arm.pattern.variant)
        .collect();

    let [entry_action, exit_action] = [
        (format_ident!("call_entry_action"), entry_actions),
        (format_ident!("call_exit_action"), exit_actions),
    ]
    .map(|(method, actions)| {
        let mount_arms = mounts
            .iter()
            .map(|mount| state_lifecycle_arm(name, mount, &method));

        lifecycle_method(
            name,
            shared_storage,
            context_type,
            &method,
            actions.as_ref(),
            &leaves,
            mount_arms.collect(),
        )
    });

    let parents = parents.as_ref().map(|parents| {
        let lifted = mounts
            .iter()
            .map(|mount| lifted_state_parent_arm(name, &parents.superstate.name, mount));

        parent_method(name, parents, lifted.collect())
    });

    let same_state = (!mounts.is_empty()).then(|| state_same_state_method(name, dispatch, mounts));
    let generics = with_mount_bounds(generics, mounts);
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

            #same_state
        }
    }
}

/// Generate the arm of `State::superstate` for `mount`, which lifts the
/// superstate of the child's state into the mount's superstate variant, or
/// returns the mount node if the child's state is at the child's top level.
fn lifted_state_parent_arm(enum_name: &Ident, superstate: &Ident, mount: &MountIr) -> TokenStream {
    let variant = &mount.variant;
    let child_state = child_state_trait(&mount.machine);

    quote! {
        #enum_name::#variant(state) => ::core::option::Option::Some(#superstate::#variant(
            match #child_state::superstate(state) {
                ::core::option::Option::Some(superstate) => {
                    ::sotu::SubmachineNode::Inner(superstate)
                }
                ::core::option::Option::None => ::sotu::SubmachineNode::Mount,
            },
        )),
    }
}

/// Generate `State::same_state` for a machine with mounts.
///
/// Leaf states compare by variant, and two states of the same mount are the
/// same state exactly when the child's states are.
fn state_same_state_method(
    enum_name: &Ident,
    dispatch: &DispatchIr,
    mounts: &[MountIr],
) -> TokenStream {
    let leaf_arms = dispatch.arms.iter().map(|arm| {
        let variant = &arm.pattern.variant;

        quote! {
            (#enum_name::#variant { .. }, #enum_name::#variant { .. }) => true,
        }
    });

    let mount_arms = mounts.iter().map(|mount| {
        let variant = &mount.variant;
        let child_state = child_state_trait(&mount.machine);

        quote! {
            (#enum_name::#variant(first), #enum_name::#variant(second)) => {
                #child_state::same_state(first, second)
            }
        }
    });

    // With a single variant the pairs above are exhaustive, and a wildcard arm
    // would be reported as unreachable in the user's crate.
    let different_variants = (dispatch.arms.len() + mounts.len() > 1).then(|| quote!(_ => false,));

    quote! {
        fn same_state(&self, other: &Self) -> bool {
            match (self, other) {
                #(#leaf_arms)*
                #(#mount_arms)*
                #different_variants
            }
        }
    }
}

/// Return the qualified path to the blocking `State` trait of the child
/// `machine`'s state type, through which every call into a child state goes.
fn child_state_trait(machine: &Type) -> TokenStream {
    quote! {
        <<#machine as ::sotu::blocking::IntoStateMachine>::State
            as ::sotu::blocking::State<#machine>>
    }
}

/// Return the qualified path to the blocking `Superstate` trait of the child
/// `machine`'s superstate type, through which every call into a child
/// superstate goes.
fn child_superstate_trait(machine: &Type) -> TokenStream {
    quote! {
        <<#machine as ::sotu::blocking::IntoStateMachine>::Superstate<'_>
            as ::sotu::blocking::Superstate<#machine>>
    }
}

/// Return `generics` with the bounds that every mounted machine must satisfy.
///
/// Each bound is spanned at the value of the `machine` key, so a child that
/// does not satisfy it is reported there.
fn with_mount_bounds(generics: &Generics, mounts: &[MountIr]) -> Generics {
    let mut generics = generics.clone();

    for mount in mounts {
        let machine = &mount.machine;
        let predicates = &mut generics.make_where_clause().predicates;

        predicates.push(parse_quote_spanned! {machine.span()=>
            #machine: ::sotu::private::BlockingSubmachine
        });

        // Without an exit handler, the parent has nowhere to route an exit of
        // the child, so the child must not be able to exit.
        predicates.push(parse_quote_spanned! {machine.span()=>
            #machine: ::sotu::private::ExitlessSubmachine
        });
    }

    generics
}

// MARK: Superstate Dispatch

fn superstate_dispatch_impl(superstate: &SuperstateIr, mounts: &[MountIr]) -> TokenStream {
    let SuperstateIr {
        ty,
        visibility: _,
        derives: _,
        variants,
        shared_storage,
        generics,
        event_type,
        context_type,
        exit_type,
        state,
        constructor_generics: _,
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
    let state = state_type(state);
    let mut generics = generics.clone();

    if has_lifetime(ty) {
        generics.params.insert(0, parse_quote!('sub));
    }

    let generics = with_mount_bounds(&generics, mounts);
    let (impl_generics, _, where_clause) = generics.split_for_impl();

    let mount_arms = mounts
        .iter()
        .map(|mount| mount_handler_arms(name, shared_storage, shared_storage_parameter, mount));

    let arms = arms
        .iter()
        .map(|arm| dispatch_arm(name, shared_storage, arm))
        .chain(mount_arms);

    let composites: Vec<&Ident> = variants.iter().map(|variant| &variant.name).collect();

    let [entry_action, exit_action] = [
        (format_ident!("call_entry_action"), entry_actions),
        (format_ident!("call_exit_action"), exit_actions),
    ]
    .map(|(method, actions)| {
        let mount_arms = mounts
            .iter()
            .map(|mount| superstate_lifecycle_arms(name, mount, &method));

        lifecycle_method(
            name,
            shared_storage,
            context_type,
            &method,
            actions.as_ref(),
            &composites,
            mount_arms.collect(),
        )
    });

    let lifted = mounts
        .iter()
        .map(|mount| lifted_superstate_parent_arms(name, mount));

    let parents = parent_method(name, parents, lifted.collect());

    let same_variant_arms = variants.iter().map(|variant| {
        let variant_name = &variant.name;

        quote! {
            (#name::#variant_name { .. }, #name::#variant_name { .. }) => true,
        }
    });

    let same_mount_arms = mounts
        .iter()
        .map(|mount| superstate_same_mount_arms(name, mount));

    // With a single variant the pairs above are exhaustive, and a wildcard arm
    // would be reported as unreachable in the user's crate. A mount variant
    // pairs its mount node with the child's superstates, which the pairs above
    // leave out.
    let different_variants =
        (variants.len() > 1 || !mounts.is_empty()).then(|| quote!(_ => false,));

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
                    #(#same_mount_arms)*
                    #different_variants
                }
            }
        }
    }
}

/// Generate the arms of `Superstate::call_handler` for `mount`.
///
/// The child's superstates defer every event, since events reach the child
/// only through the mount node, and the mount node calls the mount handler.
fn mount_handler_arms(
    enum_name: &Ident,
    shared_storage: &Type,
    shared_storage_parameter: &Ident,
    mount: &MountIr,
) -> TokenStream {
    let variant = &mount.variant;
    let handler = call(shared_storage, &mount.handler);

    // A mount handler taking the `submachine` input borrows the child's storage,
    // which is absent only when user code emptied an active instance field.
    // The event then passes on to the mount's superstates.
    let mount_call = if mount.takes_submachine {
        let field = &mount.field;
        let submachine = generated_ident("submachine");
        let storage = quote_spanned! {field.span()=>
            &mut #shared_storage_parameter.#field
        };
        let machine = &mount.machine;

        quote! {
            match ::sotu::private::MountField::<#machine>::get_mut(#storage) {
                ::core::option::Option::Some(#submachine) => #handler,
                ::core::option::Option::None => ::sotu::Outcome::Super,
            }
        }
    } else {
        handler
    };

    quote! {
        #enum_name::#variant(::sotu::SubmachineNode::Inner(_)) => ::sotu::Outcome::Super,
        #enum_name::#variant(::sotu::SubmachineNode::Mount) => #mount_call,
    }
}

/// Generate the arms of `Superstate::superstate` for `mount`.
///
/// The mount node returns its own parent. A superstate of the child returns
/// its own superstate lifted into the mount's variant, or the mount node if it
/// is at the child's top level.
fn lifted_superstate_parent_arms(enum_name: &Ident, mount: &MountIr) -> TokenStream {
    let variant = &mount.variant;
    let parent = parent_value(enum_name, mount.parent.as_ref());
    let child_superstate = child_superstate_trait(&mount.machine);

    quote! {
        #enum_name::#variant(::sotu::SubmachineNode::Mount) => #parent,
        #enum_name::#variant(::sotu::SubmachineNode::Inner(superstate)) => {
            ::core::option::Option::Some(#enum_name::#variant(
                match #child_superstate::superstate(superstate) {
                    ::core::option::Option::Some(parent) => {
                        ::sotu::SubmachineNode::Inner(parent)
                    }
                    ::core::option::Option::None => ::sotu::SubmachineNode::Mount,
                },
            ))
        }
    }
}

/// Generate the arms of `Superstate::same_state` for `mount`.
///
/// The mount node is the same node as itself, and two superstates of the child
/// are the same node exactly when the child says so.
fn superstate_same_mount_arms(enum_name: &Ident, mount: &MountIr) -> TokenStream {
    let variant = &mount.variant;
    let child_superstate = child_superstate_trait(&mount.machine);

    quote! {
        (
            #enum_name::#variant(::sotu::SubmachineNode::Mount),
            #enum_name::#variant(::sotu::SubmachineNode::Mount),
        ) => true,
        (
            #enum_name::#variant(::sotu::SubmachineNode::Inner(first)),
            #enum_name::#variant(::sotu::SubmachineNode::Inner(second)),
        ) => #child_superstate::same_state(first, second),
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

/// Generate `superstate`, matching the variants of `parents` and then the
/// arms in `mount_arms`.
fn parent_method(
    enum_name: &Ident,
    parents: &ParentDispatchIr,
    mount_arms: Vec<TokenStream>,
) -> TokenStream {
    let elided = superstate_type(&parents.superstate, &quote!('_));

    let arms = parents
        .arms
        .iter()
        .map(|arm| parent_arm(enum_name, &parents.superstate.name, arm))
        .chain(mount_arms);

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

    let parent = parent_value(superstate, arm.parent.as_ref());

    quote! {
        #pattern => #parent,
    }
}

/// Generate the optional superstate value that `parent` describes.
fn parent_value(superstate: &Ident, parent: Option<&ParentIr>) -> TokenStream {
    match parent {
        Some(ParentIr {
            variant,
            is_mount: true,
            ..
        }) => quote! {
            ::core::option::Option::Some(#superstate::#variant(::sotu::SubmachineNode::Mount))
        },
        Some(ParentIr {
            variant,
            fields,
            is_mount: false,
        }) => {
            let fields = fields
                .iter()
                .map(|ParentFieldIr { name, value }| field_initializer(name, value.as_ref()));

            quote! {
                ::core::option::Option::Some(#superstate::#variant {
                    #(#fields),*
                })
            }
        }
        None => quote!(::core::option::Option::None),
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

// MARK: Mount Lifecycle

/// Generate the entry or exit action `method` of a node enum.
///
/// Without `mount_arms`, the method exists only if a node has an action.
/// Otherwise it always exists, because entering and leaving a mount creates
/// and drops the child's instance and runs the child's actions. Each of the
/// `variants` then gets an arm from `actions`, or an empty arm if no node of
/// the enum has an action.
fn lifecycle_method(
    enum_name: &Ident,
    shared_storage: &Type,
    context_type: &Type,
    method: &Ident,
    actions: Option<&ActionDispatchIr>,
    variants: &[&Ident],
    mount_arms: Vec<TokenStream>,
) -> Option<TokenStream> {
    if mount_arms.is_empty() {
        return actions.map(|actions| {
            action_method(enum_name, shared_storage, context_type, method, actions)
        });
    }

    // The mount arms use both parameters, so neither is underscored.
    let shared_storage_parameter = generated_ident("shared_storage");
    let context_parameter = generated_ident("context");

    let arms: Vec<TokenStream> = match actions {
        Some(actions) => actions
            .arms
            .iter()
            .map(|arm| action_arm(enum_name, shared_storage, arm))
            .collect(),
        None => variants
            .iter()
            .map(|variant| quote!(#enum_name::#variant { .. } => {}))
            .collect(),
    };

    Some(quote! {
        fn #method<'ctx, 'data: 'ctx>(
            &mut self,
            #shared_storage_parameter: &mut #shared_storage,
            #context_parameter: &mut #context_type,
        ) {
            match self {
                #(#arms)*
                #(#mount_arms)*
            }
        }
    })
}

/// Generate the arm of the state action `method` for `mount`, which runs the
/// same action of the child's state on the child's storage.
fn state_lifecycle_arm(enum_name: &Ident, mount: &MountIr, method: &Ident) -> TokenStream {
    let variant = &mount.variant;
    let child_action = child_action(mount, &child_state_trait(&mount.machine), method);

    quote! {
        #enum_name::#variant(state) => #child_action,
    }
}

/// Generate the arms of the superstate action `method` for `mount`.
///
/// Entering the mount node fills an empty instance field with a new child, and
/// leaving it releases the field, so an `Option` field is `Some` exactly while
/// the mount node is active. Since the mount node is entered before and left
/// after every node of the child, the child's actions always find its storage.
/// A superstate of the child runs the same action of the child's superstate on
/// the child's storage.
fn superstate_lifecycle_arms(enum_name: &Ident, mount: &MountIr, method: &Ident) -> TokenStream {
    let variant = &mount.variant;

    let mount_node = if method == "call_entry_action" {
        field_call(mount, "DefaultInstance", "fill_default")
    } else {
        field_call(mount, "MountField", "release")
    };

    let child_action = child_action(mount, &child_superstate_trait(&mount.machine), method);

    quote! {
        #enum_name::#variant(::sotu::SubmachineNode::Mount) => #mount_node,
        #enum_name::#variant(::sotu::SubmachineNode::Inner(state)) => #child_action,
    }
}

/// Generate the call of the child's action `method` through `child_trait` on
/// the node bound as `state`, with the child's storage and the parent's
/// context.
///
/// The storage is absent only when user code emptied an active instance
/// field, and the action is then skipped.
fn child_action(mount: &MountIr, child_trait: &TokenStream, method: &Ident) -> TokenStream {
    let submachine = generated_ident("submachine");
    let context = generated_ident("context");
    let storage = field_call(mount, "MountField", "get_mut");

    quote! {
        if let ::core::option::Option::Some(#submachine) = #storage {
            #child_trait::#method(state, #submachine, #context);
        }
    }
}

/// Generate the call of `::sotu::private::<trait_name>::<C>::<method>` on the
/// mount's field of the shared storage.
///
/// Every token of the call is spanned at the value of the `field` key, so a
/// field of the wrong type is reported there.
fn field_call(mount: &MountIr, trait_name: &str, method: &str) -> TokenStream {
    let MountIr { machine, field, .. } = mount;
    let shared_storage = generated_ident("shared_storage");
    let trait_name = Ident::new(trait_name, field.span());
    let method = Ident::new(method, field.span());

    quote_spanned! {field.span()=>
        ::sotu::private::#trait_name::<#machine>::#method(&mut #shared_storage.#field)
    }
}
