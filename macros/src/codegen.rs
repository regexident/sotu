//! Turn the lowered machine into tokens.

use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    Attribute, Expr, GenericParam, Generics, Path, PathArguments, Token, Type, parse_quote,
    parse_quote_spanned,
};

use crate::analyze::{HooksModel, Mode};
use crate::lower::{
    ActionArmIr, ActionDispatchIr, ArmIr, CallIr, DispatchIr, FieldIr, Ir, MachineIr,
    MountActionIr, MountIr, NodeTypeIr, ParentArmIr, ParentDispatchIr, ParentFieldIr, ParentIr,
    PatternIr, StateEnumIr, SuperstateIr, generated_ident,
};

// MARK: Generation

/// Generate the impl block, the node enums and the trait implementations of a machine.
///
/// The traits are those of `sotu::blocking` or of `sotu::awaitable`, as
/// [`Ir::mode`] selects. A blocking machine calls its mounted children through
/// the blocking traits and so can mount blocking children only. An awaitable
/// machine calls them through `AwaitableSubmachine` and awaits every call, so
/// it can mount children of either mode.
pub(crate) fn generate(ir: &Ir) -> TokenStream {
    let mode = ir.mode;
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

    let into_state_machine = into_state_machine_impl(&ir.machine, mode);
    let dispatch = dispatch_impl(&ir.dispatch, mounts, mode);

    let superstate_dispatch = ir
        .superstate
        .as_ref()
        .map(|superstate| superstate_dispatch_impl(superstate, mounts, mode));

    let submachine = submachine_impls(&ir.machine, mode);
    let boundary = awaitable_boundary_impl(ir);
    let projection = projection_impl(&ir.machine, mounts, mode);

    quote! {
        #item

        #state_enum

        #superstate_enum

        #into_state_machine

        #dispatch

        #superstate_dispatch

        #submachine

        #boundary

        #projection
    }
}

// MARK: Modes

/// Return the module of `sotu` whose traits a machine of `mode` implements.
fn mode_module(mode: Mode) -> TokenStream {
    match mode {
        Mode::Blocking => quote!(::sotu::blocking),
        Mode::Awaitable => quote!(::sotu::awaitable),
    }
}

/// Generate the return type of a trait method whose blocking form returns
/// `output`, or nothing for `None`.
///
/// An awaitable method returns a future of the same output instead.
fn method_output(mode: Mode, output: Option<&TokenStream>) -> Option<TokenStream> {
    match (mode, output) {
        (Mode::Blocking, Some(output)) => Some(quote!(-> #output)),
        (Mode::Blocking, None) => None,
        (Mode::Awaitable, Some(output)) => {
            Some(quote!(-> impl ::core::future::Future<Output = #output>))
        }
        (Mode::Awaitable, None) => Some(quote!(-> impl ::core::future::Future<Output = ()>)),
    }
}

/// Generate the body of a handler or action method from the blocking `body`.
///
/// An awaitable method wraps `body` in an `async move` block, which takes the
/// method's references along, so the returned future borrows the node, the
/// shared storage, the event and the context for as long as it runs.
fn method_body(mode: Mode, body: TokenStream) -> TokenStream {
    match mode {
        Mode::Blocking => body,
        Mode::Awaitable => quote!(async move { #body }),
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

fn into_state_machine_impl(machine: &MachineIr, mode: Mode) -> TokenStream {
    let shared_storage = &machine.shared_storage;
    let (impl_generics, _, where_clause) = machine.generics.split_for_impl();
    let items = machine_items(machine);
    let hooks = hook_methods(machine, mode);
    let module = mode_module(mode);

    quote! {
        impl #impl_generics #module::IntoStateMachine for #shared_storage #where_clause {
            #items

            #hooks
        }
    }
}

/// Generate the hook methods whose key is present, each delegating to the arity
/// adapter that accepts the hook with or without a context parameter.
///
/// An absent key generates no method, which keeps the no-op default of the trait.
/// An awaitable hook method returns the adapter's future, which is the hook's
/// own for an `async` function and a ready one for a plain function.
fn hook_methods(machine: &MachineIr, mode: Mode) -> TokenStream {
    let HooksModel {
        before_dispatch,
        after_dispatch,
        before_transition,
        after_transition,
    } = &machine.hooks;

    let before_dispatch = before_dispatch
        .as_ref()
        .map(|path| dispatch_hook(machine, &format_ident!("before_dispatch"), path, mode));

    let after_dispatch = after_dispatch
        .as_ref()
        .map(|path| dispatch_hook(machine, &format_ident!("after_dispatch"), path, mode));

    let before_transition = before_transition
        .as_ref()
        .map(|path| transition_hook(machine, &format_ident!("before_transition"), path, mode));

    let after_transition = after_transition
        .as_ref()
        .map(|path| transition_hook(machine, &format_ident!("after_transition"), path, mode));

    quote! {
        #before_dispatch

        #after_dispatch

        #before_transition

        #after_transition
    }
}

fn dispatch_hook(machine: &MachineIr, method: &Ident, path: &Path, mode: Mode) -> TokenStream {
    let MachineIr {
        event_type,
        context_type,
        state,
        superstate,
        ..
    } = machine;

    let state = state_type(state);
    let module = mode_module(mode);

    // The future that the awaitable trait method returns captures the lifetime
    // in `Self::Superstate<'_>`, and a lifetime that appears only in a
    // projection is early-bound. The impl names the superstate through the
    // same projection, so that it declares the same lifetime parameters.
    let superstate = match (mode, superstate) {
        (Mode::Awaitable, _) => quote!(<Self as #module::IntoStateMachine>::Superstate<'_>),
        (Mode::Blocking, Some(ty)) => superstate_type(ty, &quote!('_)),
        (Mode::Blocking, None) => quote!(()),
    };

    let output = method_output(mode, None);

    quote! {
        fn #method<'ctx, 'data: 'ctx>(
            &mut self,
            node: ::sotu::StateOrSuperstate<'_, #state, #superstate>,
            event: &#event_type,
            context: &mut #context_type,
        ) #output {
            #module::DispatchHook::call_dispatch_hook(&(#path), self, node, event, context)
        }
    }
}

fn transition_hook(machine: &MachineIr, method: &Ident, path: &Path, mode: Mode) -> TokenStream {
    let MachineIr {
        context_type,
        state,
        ..
    } = machine;

    let state = state_type(state);
    let module = mode_module(mode);
    let output = method_output(mode, None);

    quote! {
        fn #method<'ctx, 'data: 'ctx>(
            &mut self,
            source: &#state,
            target: &#state,
            context: &mut #context_type,
        ) #output {
            #module::TransitionHook::call_transition_hook(
                &(#path),
                self,
                source,
                target,
                context,
            )
        }
    }
}

fn submachine_impls(machine: &MachineIr, mode: Mode) -> TokenStream {
    let shared_storage = &machine.shared_storage;
    let (impl_generics, _, where_clause) = machine.generics.split_for_impl();
    let items = machine_items(machine);

    // A blocking parent requires this marker of every child, so only blocking
    // machines can be mounted in blocking ones.
    let blocking = (mode == Mode::Blocking).then(|| {
        quote! {
            impl #impl_generics ::sotu::private::BlockingSubmachine for #shared_storage
            #where_clause
            {}
        }
    });

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

        #blocking

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

// MARK: Awaitable Boundary

/// Generate the `AwaitableSubmachine` impl of the machine.
///
/// An awaitable machine always implements it. A blocking machine implements
/// it only when the macros' `async` feature is enabled, which is the case
/// exactly when `sotu` provides the awaitable boundary.
fn awaitable_boundary_impl(ir: &Ir) -> Option<TokenStream> {
    if ir.mode == Mode::Blocking && !cfg!(feature = "async") {
        return None;
    }

    let shared_storage = &ir.machine.shared_storage;
    let generics = boundary_generics(ir);
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let methods = awaitable_boundary_methods(ir);

    // The impl reads the static depth that the superstate enum provides.
    let depth = ir
        .superstate
        .as_ref()
        .map(|superstate| superstate_depth_impl(superstate, &ir.mounts));

    Some(quote! {
        impl #impl_generics ::sotu::private::AwaitableSubmachine for #shared_storage
        #where_clause
        {
            #methods
        }

        #depth
    })
}

/// Return the generics of the `AwaitableSubmachine` impl.
///
/// The impl calls the machine's own trait impls, so it needs the bounds of
/// those.
fn boundary_generics(ir: &Ir) -> Generics {
    let generics = with_mount_bounds(&ir.machine.generics, &ir.mounts, ir.mode);

    match ir.mode {
        Mode::Blocking => with_boundary_bounds(&generics, &ir.mounts),
        Mode::Awaitable => generics,
    }
}

/// Return `generics` with the bound that a parent needs to call each mounted
/// machine through the awaitable boundary.
///
/// Each bound is spanned at the value of the `machine` key, like the bounds of
/// [`with_mount_bounds`].
fn with_boundary_bounds(generics: &Generics, mounts: &[MountIr]) -> Generics {
    let mut generics = generics.clone();

    for mount in mounts {
        let machine = &mount.machine;

        generics
            .make_where_clause()
            .predicates
            .push(parse_quote_spanned! {machine.span()=>
                #machine: ::sotu::private::AwaitableSubmachine
            });
    }

    generics
}

/// Generate the private `__depth` function of the superstate enum, which
/// returns the depth of a variant without walking the chain above it.
///
/// A borrowed superstate can not be walked in awaitable mode, so the depth is
/// a property of the variant: the length of its declared chain of enclosing
/// nodes. A superstate of a mounted child is as deep as the mount node plus
/// the child's own depth of it.
fn superstate_depth_impl(superstate: &SuperstateIr, mounts: &[MountIr]) -> TokenStream {
    let name = &superstate.ty.name;
    let generics = with_boundary_bounds(&superstate.ty.generics, mounts);
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let variant_arms = superstate.variants.iter().map(|variant| {
        let variant_name = &variant.name;
        let depth = variant.depth;

        quote! {
            #name::#variant_name { .. } => #depth,
        }
    });

    let mount_arms = mounts.iter().map(|mount| {
        let MountIr {
            variant,
            machine,
            depth,
            ..
        } = mount;

        quote! {
            #name::#variant(::sotu::SubmachineNode::Mount) => #depth,
            #name::#variant(::sotu::SubmachineNode::Inner(superstate)) => {
                #depth + <#machine as ::sotu::private::AwaitableSubmachine>::superstate_depth(
                    superstate,
                )
            }
        }
    });

    quote! {
        impl #impl_generics #name #type_generics #where_clause {
            fn __depth(&self) -> usize {
                match self {
                    #(#variant_arms)*
                    #(#mount_arms)*
                }
            }
        }
    }
}

/// Return the qualified path to the `State` trait of `mode` for the state
/// type of `machine`, through which every call into a state goes.
fn state_trait(machine: &Type, mode: Mode) -> TokenStream {
    let module = mode_module(mode);

    quote! {
        <<#machine as #module::IntoStateMachine>::State as #module::State<#machine>>
    }
}

/// Return the qualified path to the `Superstate` trait of `mode` for the
/// superstate type of `machine`, through which every call into a superstate
/// goes.
fn superstate_trait(machine: &Type, mode: Mode) -> TokenStream {
    let module = mode_module(mode);

    quote! {
        <<#machine as #module::IntoStateMachine>::Superstate<'_>
            as #module::Superstate<#machine>>
    }
}

/// Generate the methods of the `AwaitableSubmachine` impl of the machine.
///
/// An awaitable machine delegates every method to its own traits. A blocking
/// machine calls its blocking traits and wraps the result of each call that
/// the boundary awaits in a future that is ready on its first poll, so a
/// blocking call runs to completion when the future is created.
fn awaitable_boundary_methods(ir: &Ir) -> TokenStream {
    let mode = ir.mode;
    let machine = &ir.machine.shared_storage;
    let module = mode_module(mode);
    let state_trait = state_trait(machine, mode);
    let superstate_trait = superstate_trait(machine, mode);

    let state_extension = quote! {
        <<#machine as #module::IntoStateMachine>::State as #module::StateExt<#machine>>
    };

    let submachine = quote!(::sotu::private::Submachine);
    let state = quote!(<Self as #submachine>::State);
    let superstate_sub = quote!(<Self as #submachine>::Superstate<'sub>);
    let event = quote!(<Self as #submachine>::Event);

    let superstate_depth = boundary_superstate_depth(ir);

    let superstate_parent = boundary_superstate_parent(ir, &superstate_trait);

    let ready = |call: TokenStream| match mode {
        Mode::Blocking => quote!(::core::future::ready(#call)),
        Mode::Awaitable => call,
    };

    let exit = quote!(<Self as #submachine>::Exit);
    let context = quote!(<Self as #submachine>::Context<'ctx, 'data>);
    let dispatched = quote!(::sotu::private::Dispatched<#state, #exit, #event>);
    let future = |output: TokenStream| quote!(impl ::core::future::Future<Output = #output>);
    let dispatched_future = future(dispatched);
    let unit_future = future(quote!(()));

    let dispatch_state = ready(quote! {
        #state_trait::__dispatch(state, shared_storage, event, context)
    });

    let dispatch_superstate = ready(quote! {
        #superstate_trait::__dispatch(superstate, shared_storage, event, context)
    });

    let enter_state = ready(quote! {
        #state_trait::call_entry_action(state, shared_storage, context)
    });

    let exit_state = ready(quote! {
        #state_trait::call_exit_action(state, shared_storage, context)
    });

    let enter_superstate = ready(quote! {
        #superstate_trait::call_entry_action(superstate, shared_storage, context)
    });

    let exit_superstate = ready(quote! {
        #superstate_trait::call_exit_action(superstate, shared_storage, context)
    });

    let machine_trait = quote!(<#machine as #module::IntoStateMachine>);

    let before_dispatch = ready(quote! {
        #machine_trait::before_dispatch(shared_storage, node, event, context)
    });

    let after_dispatch = ready(quote! {
        #machine_trait::after_dispatch(shared_storage, node, event, context)
    });

    let before_transition = ready(quote! {
        #machine_trait::before_transition(shared_storage, source, target, context)
    });

    let after_transition = ready(quote! {
        #machine_trait::after_transition(shared_storage, source, target, context)
    });

    let before_internal_transition = ready(quote! {
        #state_trait::__before_internal_transition(
            source,
            target,
            exit_levels,
            nesting,
            shared_storage,
            context,
        )
    });

    let after_internal_transition = ready(quote! {
        #state_trait::__after_internal_transition(
            source,
            target,
            exit_levels,
            nesting,
            shared_storage,
            context,
        )
    });

    quote! {
        fn state_superstate<'sub>(
            state: &'sub mut #state,
        ) -> ::core::option::Option<#superstate_sub>
        where
            Self: 'sub,
        {
            #state_trait::superstate(state)
        }

        fn state_same(first: &#state, second: &#state) -> bool {
            #state_trait::same_state(first, second)
        }

        fn superstate_same<'first, 'second>(
            first: &<Self as #submachine>::Superstate<'first>,
            second: &<Self as #submachine>::Superstate<'second>,
        ) -> bool
        where
            Self: 'first + 'second,
        {
            #superstate_trait::same_state(first, second)
        }

        fn state_depth(state: &mut #state) -> usize {
            #state_extension::depth(state)
        }

        fn state_is_addressed(state: &#state, event: #event) -> (bool, #event) {
            #state_trait::__is_addressed(state, event)
        }

        fn superstate_is_addressed<'sub>(
            superstate: &#superstate_sub,
            event: #event,
        ) -> (bool, #event)
        where
            Self: 'sub,
        {
            #superstate_trait::__is_addressed(superstate, event)
        }

        #superstate_depth

        #superstate_parent

        fn dispatch_state<'ctx, 'data: 'ctx>(
            state: &mut #state,
            shared_storage: &mut Self,
            event: #event,
            context: &mut #context,
        ) -> #dispatched_future {
            #dispatch_state
        }

        fn dispatch_superstate<'ctx, 'data: 'ctx, 'sub>(
            superstate: &mut #superstate_sub,
            shared_storage: &mut Self,
            event: #event,
            context: &mut #context,
        ) -> #dispatched_future
        where
            Self: 'sub,
        {
            #dispatch_superstate
        }

        fn enter_state<'ctx, 'data: 'ctx>(
            state: &mut #state,
            shared_storage: &mut Self,
            context: &mut #context,
        ) -> #unit_future {
            #enter_state
        }

        fn exit_state<'ctx, 'data: 'ctx>(
            state: &mut #state,
            shared_storage: &mut Self,
            context: &mut #context,
        ) -> #unit_future {
            #exit_state
        }

        fn enter_superstate<'ctx, 'data: 'ctx, 'sub>(
            superstate: &mut #superstate_sub,
            shared_storage: &mut Self,
            context: &mut #context,
        ) -> #unit_future
        where
            Self: 'sub,
        {
            #enter_superstate
        }

        fn exit_superstate<'ctx, 'data: 'ctx, 'sub>(
            superstate: &mut #superstate_sub,
            shared_storage: &mut Self,
            context: &mut #context,
        ) -> #unit_future
        where
            Self: 'sub,
        {
            #exit_superstate
        }

        fn before_dispatch<'ctx, 'data: 'ctx, 'sub>(
            shared_storage: &mut Self,
            node: ::sotu::StateOrSuperstate<'_, #state, #superstate_sub>,
            event: &#event,
            context: &mut #context,
        ) -> #unit_future
        where
            Self: 'sub,
        {
            #before_dispatch
        }

        fn after_dispatch<'ctx, 'data: 'ctx, 'sub>(
            shared_storage: &mut Self,
            node: ::sotu::StateOrSuperstate<'_, #state, #superstate_sub>,
            event: &#event,
            context: &mut #context,
        ) -> #unit_future
        where
            Self: 'sub,
        {
            #after_dispatch
        }

        fn before_transition<'ctx, 'data: 'ctx>(
            shared_storage: &mut Self,
            source: &#state,
            target: &#state,
            context: &mut #context,
        ) -> #unit_future {
            #before_transition
        }

        fn after_transition<'ctx, 'data: 'ctx>(
            shared_storage: &mut Self,
            source: &#state,
            target: &#state,
            context: &mut #context,
        ) -> #unit_future {
            #after_transition
        }

        fn internal_transition_nesting(
            source: &mut #state,
            target: &mut #state,
            exit_levels: usize,
        ) -> usize {
            #state_trait::__internal_transition_nesting(source, target, exit_levels)
        }

        fn before_internal_transition<'ctx, 'data: 'ctx>(
            source: &mut #state,
            target: &mut #state,
            exit_levels: usize,
            nesting: usize,
            shared_storage: &mut Self,
            context: &mut #context,
        ) -> #unit_future {
            #before_internal_transition
        }

        fn after_internal_transition<'ctx, 'data: 'ctx>(
            source: &mut #state,
            target: &mut #state,
            exit_levels: usize,
            nesting: usize,
            shared_storage: &mut Self,
            context: &mut #context,
        ) -> #unit_future {
            #after_internal_transition
        }
    }
}

/// Generate `superstate_depth`, which reads the superstate enum.
///
/// A machine without one has the placeholder superstate `()`, which no state
/// returns, so its depth is `0`.
fn boundary_superstate_depth(ir: &Ir) -> TokenStream {
    let superstate = quote!(<Self as ::sotu::private::Submachine>::Superstate<'sub>);

    match &ir.superstate {
        Some(_) => quote! {
            fn superstate_depth<'sub>(superstate: &#superstate) -> usize
            where
                Self: 'sub,
            {
                superstate.__depth()
            }
        },
        None => quote! {
            fn superstate_depth<'sub>(_superstate: &#superstate) -> usize
            where
                Self: 'sub,
            {
                0
            }
        },
    }
}

/// Generate `superstate_parent`.
///
/// An awaitable machine delegates to its consuming `Superstate::superstate`.
/// A blocking machine builds the parent from the fields of the consumed
/// superstate, or has none without a superstate enum.
fn boundary_superstate_parent(ir: &Ir, superstate_trait: &TokenStream) -> TokenStream {
    let superstate = quote!(<Self as ::sotu::private::Submachine>::Superstate<'sub>);

    let (parameter, body) = match (&ir.superstate, ir.mode) {
        (Some(superstate_ir), Mode::Blocking) => (
            quote!(superstate),
            consuming_parent_body(superstate_ir, &ir.mounts),
        ),
        (None, Mode::Blocking) => (quote!(_superstate), quote!(::core::option::Option::None)),
        (_, Mode::Awaitable) => (
            quote!(superstate),
            quote!(#superstate_trait::superstate(superstate)),
        ),
    };

    quote! {
        fn superstate_parent<'sub>(
            #parameter: #superstate,
        ) -> ::core::option::Option<#superstate>
        where
            Self: 'sub,
        {
            #body
        }
    }
}

/// Generate the body that consumes a blocking superstate and returns its
/// parent, moving the borrowed fields into the parent.
///
/// A superstate of a mounted child asks the child through the boundary for
/// the parent of the child's superstate and lifts it into the mount's variant.
fn consuming_parent_body(superstate: &SuperstateIr, mounts: &[MountIr]) -> TokenStream {
    let name = &superstate.ty.name;
    let parents = &superstate.consuming_parents;

    let arms = parents
        .arms
        .iter()
        .map(|arm| parent_arm(name, &parents.superstate.name, arm));

    let lifted = lifted_consuming_parent_arms(name, mounts);

    quote! {
        match superstate {
            #(#arms)*
            #(#lifted)*
        }
    }
}

/// Generate the arms of a consuming `superstate` for `mounts`, each of which
/// asks the child through the awaitable boundary for the parent of the child's
/// superstate.
fn lifted_consuming_parent_arms(enum_name: &Ident, mounts: &[MountIr]) -> Vec<TokenStream> {
    mounts
        .iter()
        .map(|mount| {
            lifted_superstate_parent_arms(
                enum_name,
                mount,
                &child_call(&mount.machine, Mode::Awaitable, ChildCall::SuperstateParent),
            )
        })
        .collect()
}

// MARK: State Dispatch

fn dispatch_impl(dispatch: &DispatchIr, mounts: &[MountIr], mode: Mode) -> TokenStream {
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

    // Dispatch reaches the child's states through `__dispatch`, which projects
    // the event into the child, so `call_handler` of a mount variant only defers.
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
            .map(|mount| state_lifecycle_arm(name, mount, &method, mode));

        lifecycle_method(
            name,
            shared_storage,
            context_type,
            &method,
            actions.as_ref(),
            &leaves,
            mount_arms.collect(),
            mode,
        )
    });

    let parents = parents.as_ref().map(|parents| {
        let lifted = mounts
            .iter()
            .map(|mount| lifted_state_parent_arm(name, &parents.superstate.name, mount, mode));

        parent_method(name, parents, lifted.collect())
    });

    let same_state =
        (!mounts.is_empty()).then(|| state_same_state_method(name, dispatch, mounts, mode));

    let boundary = (!mounts.is_empty()).then(|| state_boundary_methods(dispatch, mounts, mode));

    let internal_transition =
        (!mounts.is_empty()).then(|| internal_transition_methods(dispatch, mounts, mode));

    let generics = with_mount_bounds(generics, mounts, mode);
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let module = mode_module(mode);
    let output = method_output(mode, Some(&quote!(::sotu::Outcome<Self, #exit_type>)));

    let body = method_body(
        mode,
        quote! {
            match self {
                #(#arms)*
            }
        },
    );

    quote! {
        impl #impl_generics #module::State<#shared_storage> for #state #where_clause {
            fn call_handler<'ctx, 'data: 'ctx>(
                &mut self,
                #shared_storage_parameter: &mut #shared_storage,
                #event_parameter: &#event_type,
                #context_parameter: &mut #context_type,
            ) #output {
                #body
            }

            #entry_action

            #exit_action

            #parents

            #same_state

            #boundary

            #internal_transition
        }
    }
}

/// Generate the arm of `State::superstate` for `mount`, which lifts the
/// superstate of the child's state into the mount's superstate variant, or
/// returns the mount node if the child's state is at the child's top level.
fn lifted_state_parent_arm(
    enum_name: &Ident,
    superstate: &Ident,
    mount: &MountIr,
    mode: Mode,
) -> TokenStream {
    let variant = &mount.variant;
    let child_parent = child_call(&mount.machine, mode, ChildCall::StateSuperstate);

    quote! {
        #enum_name::#variant(state) => ::core::option::Option::Some(#superstate::#variant(
            match #child_parent(state) {
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
    mode: Mode,
) -> TokenStream {
    let leaf_arms = dispatch.arms.iter().map(|arm| {
        let variant = &arm.pattern.variant;

        quote! {
            (#enum_name::#variant { .. }, #enum_name::#variant { .. }) => true,
        }
    });

    let mount_arms = mounts.iter().map(|mount| {
        let variant = &mount.variant;
        let child_same = child_call(&mount.machine, mode, ChildCall::StateSame);

        quote! {
            (#enum_name::#variant(first), #enum_name::#variant(second)) => {
                #child_same(first, second)
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

// MARK: Child Calls

/// A function through which a parent calls into one of its mounted children.
///
/// A blocking parent calls the blocking traits of the child, and an awaitable
/// parent calls `AwaitableSubmachine`. A call takes the same arguments in both
/// modes, so a call site differs between them only by the `.await` that
/// [`awaited`] returns.
#[derive(Clone, Copy)]
enum ChildCall {
    /// The superstate enclosing a child state, borrowing from the state.
    StateSuperstate,
    /// The superstate enclosing a child superstate, which a blocking parent
    /// borrows and an awaitable one consumes.
    SuperstateParent,
    StateSame,
    SuperstateSame,
    StateDepth,
    SuperstateDepth,
    StateIsAddressed,
    SuperstateIsAddressed,
    StateDispatch,
    SuperstateDispatch,
    StateEntry,
    StateExit,
    SuperstateEntry,
    SuperstateExit,
    BeforeDispatch,
    AfterDispatch,
    BeforeTransition,
    AfterTransition,
    InternalTransitionNesting,
    BeforeInternalTransition,
    AfterInternalTransition,
}

/// Return the path of the function that `call` names for the child `machine`
/// in a parent of `mode`.
fn child_call(machine: &Type, mode: Mode, call: ChildCall) -> TokenStream {
    let state_trait = state_trait(machine, Mode::Blocking);
    let superstate_trait = superstate_trait(machine, Mode::Blocking);
    let machine_trait = quote!(<#machine as ::sotu::blocking::IntoStateMachine>);

    let state_extension = quote! {
        <<#machine as ::sotu::blocking::IntoStateMachine>::State
            as ::sotu::blocking::StateExt<#machine>>
    };

    let superstate_extension = quote! {
        <<#machine as ::sotu::blocking::IntoStateMachine>::Superstate<'_>
            as ::sotu::blocking::SuperstateExt<#machine>>
    };

    let (blocking, awaitable) = match call {
        ChildCall::StateSuperstate => (quote!(#state_trait::superstate), "state_superstate"),
        ChildCall::SuperstateParent => (quote!(#superstate_trait::superstate), "superstate_parent"),
        ChildCall::StateSame => (quote!(#state_trait::same_state), "state_same"),
        ChildCall::SuperstateSame => (quote!(#superstate_trait::same_state), "superstate_same"),
        ChildCall::StateDepth => (quote!(#state_extension::depth), "state_depth"),
        ChildCall::SuperstateDepth => (quote!(#superstate_extension::depth), "superstate_depth"),
        ChildCall::StateIsAddressed => (quote!(#state_trait::__is_addressed), "state_is_addressed"),
        ChildCall::SuperstateIsAddressed => (
            quote!(#superstate_trait::__is_addressed),
            "superstate_is_addressed",
        ),
        ChildCall::StateDispatch => (quote!(#state_trait::__dispatch), "dispatch_state"),
        ChildCall::SuperstateDispatch => {
            (quote!(#superstate_trait::__dispatch), "dispatch_superstate")
        }
        ChildCall::StateEntry => (quote!(#state_trait::call_entry_action), "enter_state"),
        ChildCall::StateExit => (quote!(#state_trait::call_exit_action), "exit_state"),
        ChildCall::SuperstateEntry => (
            quote!(#superstate_trait::call_entry_action),
            "enter_superstate",
        ),
        ChildCall::SuperstateExit => (
            quote!(#superstate_trait::call_exit_action),
            "exit_superstate",
        ),
        ChildCall::BeforeDispatch => (quote!(#machine_trait::before_dispatch), "before_dispatch"),
        ChildCall::AfterDispatch => (quote!(#machine_trait::after_dispatch), "after_dispatch"),
        ChildCall::BeforeTransition => (
            quote!(#machine_trait::before_transition),
            "before_transition",
        ),
        ChildCall::AfterTransition => {
            (quote!(#machine_trait::after_transition), "after_transition")
        }
        ChildCall::InternalTransitionNesting => (
            quote!(#state_trait::__internal_transition_nesting),
            "internal_transition_nesting",
        ),
        ChildCall::BeforeInternalTransition => (
            quote!(#state_trait::__before_internal_transition),
            "before_internal_transition",
        ),
        ChildCall::AfterInternalTransition => (
            quote!(#state_trait::__after_internal_transition),
            "after_internal_transition",
        ),
    };

    match mode {
        Mode::Blocking => blocking,
        Mode::Awaitable => awaitable_call(machine, awaitable),
    }
}

/// Return the path of the function `name` of `AwaitableSubmachine` for the
/// child `machine`.
fn awaitable_call(machine: &Type, name: &str) -> TokenStream {
    let name = format_ident!("{name}");

    quote!(<#machine as ::sotu::private::AwaitableSubmachine>::#name)
}

/// Return the `.await` that follows a call into a child in an awaitable
/// parent, or nothing in a blocking one.
fn awaited(mode: Mode) -> Option<TokenStream> {
    (mode == Mode::Awaitable).then(|| quote!(.await))
}

/// Return `generics` with the bounds that every mounted machine must satisfy.
///
/// Each bound is spanned at the value of the `machine` key, so a child that
/// does not satisfy it is reported there.
fn with_mount_bounds(generics: &Generics, mounts: &[MountIr], mode: Mode) -> Generics {
    let mut generics = generics.clone();

    for mount in mounts {
        let machine = &mount.machine;
        let predicates = &mut generics.make_where_clause().predicates;

        predicates.push(match mode {
            Mode::Blocking => parse_quote_spanned! {machine.span()=>
                #machine: ::sotu::private::BlockingSubmachine
            },
            Mode::Awaitable => parse_quote_spanned! {machine.span()=>
                #machine: ::sotu::private::AwaitableSubmachine
            },
        });

        // Without an exit handler, the parent has nowhere to route an exit of
        // the child, so the child must not be able to exit.
        if mount.exit_handler.is_none() {
            predicates.push(parse_quote_spanned! {machine.span()=>
                #machine: ::sotu::private::ExitlessSubmachine
            });
        }
    }

    generics
}

// MARK: Superstate Dispatch

fn superstate_dispatch_impl(
    superstate: &SuperstateIr,
    mounts: &[MountIr],
    mode: Mode,
) -> TokenStream {
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
        consuming_parents: _,
    } = superstate;

    let name = &ty.name;
    let declared = superstate_type(ty, &quote!('sub));
    let elided = superstate_type(ty, &quote!('_));
    let state = state_type(state);
    let mut generics = generics.clone();

    if has_lifetime(ty) {
        generics.params.insert(0, parse_quote!('sub));
    }

    let generics = with_mount_bounds(&generics, mounts, mode);
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
            .map(|mount| superstate_lifecycle_arms(name, shared_storage, mount, &method, mode));

        lifecycle_method(
            name,
            shared_storage,
            context_type,
            &method,
            actions.as_ref(),
            &composites,
            mount_arms.collect(),
            mode,
        )
    });

    let parents = match mode {
        Mode::Blocking => {
            let lifted = mounts.iter().map(|mount| {
                let child_parent =
                    child_call(&mount.machine, Mode::Blocking, ChildCall::SuperstateParent);

                lifted_superstate_parent_arms(name, mount, &child_parent)
            });

            parent_method(name, parents, lifted.collect())
        }
        Mode::Awaitable => consuming_parent_method(name, parents, mounts),
    };

    let same_variant_arms = variants.iter().map(|variant| {
        let variant_name = &variant.name;

        quote! {
            (#name::#variant_name { .. }, #name::#variant_name { .. }) => true,
        }
    });

    let same_mount_arms = mounts
        .iter()
        .map(|mount| superstate_same_mount_arms(name, mount, mode));

    // With a single variant the pairs above are exhaustive, and a wildcard arm
    // would be reported as unreachable in the user's crate. A mount variant
    // pairs its mount node with the child's superstates, which the pairs above
    // leave out.
    let different_variants =
        (variants.len() > 1 || !mounts.is_empty()).then(|| quote!(_ => false,));

    let boundary =
        (!mounts.is_empty()).then(|| superstate_boundary_methods(superstate, mounts, mode));

    let module = mode_module(mode);
    let output = method_output(mode, Some(&quote!(::sotu::Outcome<#state, #exit_type>)));

    let body = method_body(
        mode,
        quote! {
            match self {
                #(#arms)*
            }
        },
    );

    quote! {
        impl #impl_generics #module::Superstate<#shared_storage> for #declared
        #where_clause
        {
            fn call_handler<'ctx, 'data: 'ctx>(
                &mut self,
                #shared_storage_parameter: &mut #shared_storage,
                #event_parameter: &#event_type,
                #context_parameter: &mut #context_type,
            ) #output {
                #body
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

            #boundary
        }
    }
}

/// Generate the arms of `Superstate::call_handler` for `mount`.
///
/// The child's superstates defer every event, since dispatch reaches them
/// through `__dispatch`, which projects the event into the child, and the
/// mount node calls the mount handler.
fn mount_handler_arms(
    enum_name: &Ident,
    shared_storage: &Type,
    shared_storage_parameter: &Ident,
    mount: &MountIr,
) -> TokenStream {
    let variant = &mount.variant;
    let handler = call(shared_storage, &mount.handler);

    // Without the instance, the event passes on to the mount's superstates.
    let mount_call = if mount.takes_submachine {
        instance_call(
            mount,
            shared_storage_parameter,
            &handler,
            &quote!(::sotu::Outcome::Super),
        )
    } else {
        handler
    };

    quote! {
        #enum_name::#variant(::sotu::SubmachineNode::Inner(_)) => ::sotu::Outcome::Super,
        #enum_name::#variant(::sotu::SubmachineNode::Mount) => #mount_call,
    }
}

/// Generate `call` with the `submachine` input bound to the mount's instance,
/// borrowed from the shared storage named `shared_storage_parameter`.
///
/// The instance is absent only when user code emptied an active instance
/// field, and the expression is then `vacant` instead.
fn instance_call(
    mount: &MountIr,
    shared_storage_parameter: &Ident,
    call: &TokenStream,
    vacant: &TokenStream,
) -> TokenStream {
    let field = &mount.field;
    let submachine = generated_ident("submachine");
    let storage = quote_spanned! {field.span()=>
        &mut #shared_storage_parameter.#field
    };
    let machine = &mount.machine;

    quote! {
        match ::sotu::private::MountField::<#machine>::get_mut(#storage) {
            ::core::option::Option::Some(#submachine) => #call,
            ::core::option::Option::None => #vacant,
        }
    }
}

/// Generate the arms of `Superstate::superstate` for `mount`.
///
/// The mount node returns its own parent. A superstate of the child returns
/// the parent that `child_parent` computes from it, lifted into the mount's
/// variant, or the mount node if it is at the child's top level.
fn lifted_superstate_parent_arms(
    enum_name: &Ident,
    mount: &MountIr,
    child_parent: &TokenStream,
) -> TokenStream {
    let variant = &mount.variant;
    let parent = parent_value(enum_name, mount.parent.as_ref());

    quote! {
        #enum_name::#variant(::sotu::SubmachineNode::Mount) => #parent,
        #enum_name::#variant(::sotu::SubmachineNode::Inner(superstate)) => {
            ::core::option::Option::Some(#enum_name::#variant(
                match #child_parent(superstate) {
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
fn superstate_same_mount_arms(enum_name: &Ident, mount: &MountIr, mode: Mode) -> TokenStream {
    let variant = &mount.variant;
    let child_same = child_call(&mount.machine, mode, ChildCall::SuperstateSame);

    quote! {
        (
            #enum_name::#variant(::sotu::SubmachineNode::Mount),
            #enum_name::#variant(::sotu::SubmachineNode::Mount),
        ) => true,
        (
            #enum_name::#variant(::sotu::SubmachineNode::Inner(first)),
            #enum_name::#variant(::sotu::SubmachineNode::Inner(second)),
        ) => #child_same(first, second),
    }
}

// MARK: Shared Dispatch

fn action_method(
    enum_name: &Ident,
    shared_storage: &Type,
    context_type: &Type,
    method: &Ident,
    actions: &ActionDispatchIr,
    mode: Mode,
) -> TokenStream {
    let shared_storage_parameter = &actions.shared_storage_parameter;
    let context_parameter = &actions.context_parameter;

    let arms = actions
        .arms
        .iter()
        .map(|arm| action_arm(enum_name, shared_storage, arm));

    let output = method_output(mode, None);

    let body = method_body(
        mode,
        quote! {
            match self {
                #(#arms)*
            }
        },
    );

    quote! {
        fn #method<'ctx, 'data: 'ctx>(
            &mut self,
            #shared_storage_parameter: &mut #shared_storage,
            #context_parameter: &mut #context_type,
        ) #output {
            #body
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

/// Generate the consuming `superstate` of an awaitable superstate enum, which
/// moves the fields that the parent borrows from `self` into the parent.
///
/// A mount variant asks the child for the parent of the child's superstate, so
/// that the borrows of the child's superstate move into the lifted parent too.
fn consuming_parent_method(
    enum_name: &Ident,
    parents: &ParentDispatchIr,
    mounts: &[MountIr],
) -> TokenStream {
    let arms = parents
        .arms
        .iter()
        .map(|arm| parent_arm(enum_name, &parents.superstate.name, arm));

    let lifted = lifted_consuming_parent_arms(enum_name, mounts);

    quote! {
        fn superstate(self) -> ::core::option::Option<Self> {
            match self {
                #(#arms)*
                #(#lifted)*
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
        is_async,
    } = call;

    let path = expression_path(shared_storage);

    // Only an awaitable machine has `async` functions, and its calls sit in
    // the `async move` blocks of its methods.
    let awaited = is_async.then(|| quote!(.await));

    quote! {
        #path::#function(#(#arguments),*)#awaited
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
/// the enum has an action. An awaitable method awaits the action of every arm.
// Each argument comes from a different part of the lowered machine, and a
// struct bundling them would be built at the two call sites alone.
#[allow(clippy::too_many_arguments)]
fn lifecycle_method(
    enum_name: &Ident,
    shared_storage: &Type,
    context_type: &Type,
    method: &Ident,
    actions: Option<&ActionDispatchIr>,
    variants: &[&Ident],
    mount_arms: Vec<TokenStream>,
    mode: Mode,
) -> Option<TokenStream> {
    if mount_arms.is_empty() {
        return actions.map(|actions| {
            action_method(
                enum_name,
                shared_storage,
                context_type,
                method,
                actions,
                mode,
            )
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

    let output = method_output(mode, None);

    let body = method_body(
        mode,
        quote! {
            match self {
                #(#arms)*
                #(#mount_arms)*
            }
        },
    );

    Some(quote! {
        fn #method<'ctx, 'data: 'ctx>(
            &mut self,
            #shared_storage_parameter: &mut #shared_storage,
            #context_parameter: &mut #context_type,
        ) #output {
            #body
        }
    })
}

/// Generate the arm of the state action `method` for `mount`, which runs the
/// same action of the child's state on the child's storage.
fn state_lifecycle_arm(
    enum_name: &Ident,
    mount: &MountIr,
    method: &Ident,
    mode: Mode,
) -> TokenStream {
    let variant = &mount.variant;

    let call = if method == "call_entry_action" {
        ChildCall::StateEntry
    } else {
        ChildCall::StateExit
    };

    let child_action = child_action(mount, call, mode);

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
/// The mount's own entry action runs after the instance is filled and its exit
/// action before the instance is released, so both can borrow it. A superstate
/// of the child runs the same action of the child's superstate on the child's
/// storage.
fn superstate_lifecycle_arms(
    enum_name: &Ident,
    shared_storage: &Type,
    mount: &MountIr,
    method: &Ident,
    mode: Mode,
) -> TokenStream {
    let variant = &mount.variant;

    let is_entry = method == "call_entry_action";

    let instance = if is_entry {
        match &mount.factory {
            Some(factory) => factory_call(shared_storage, mount, factory),
            None => field_call(mount, "DefaultInstance", "fill_default"),
        }
    } else {
        field_call(mount, "MountField", "release")
    };

    let action = if is_entry {
        &mount.entry_action
    } else {
        &mount.exit_action
    };

    let mount_node = match action {
        Some(action) => {
            let action = mount_action(shared_storage, mount, action);

            let steps = if is_entry {
                [&instance, &action]
            } else {
                [&action, &instance]
            };

            quote! {
                {
                    #(#steps;)*
                }
            }
        }
        None => instance,
    };

    let call = if is_entry {
        ChildCall::SuperstateEntry
    } else {
        ChildCall::SuperstateExit
    };

    let child_action = child_action(mount, call, mode);

    quote! {
        #enum_name::#variant(::sotu::SubmachineNode::Mount) => #mount_node,
        #enum_name::#variant(::sotu::SubmachineNode::Inner(state)) => #child_action,
    }
}

/// Generate the call of the mount's `action`, which is skipped if it takes the
/// `submachine` input and the instance is absent.
fn mount_action(shared_storage: &Type, mount: &MountIr, action: &MountActionIr) -> TokenStream {
    let call = call(shared_storage, &action.call);

    if action.takes_submachine {
        instance_call(
            mount,
            &generated_ident("shared_storage"),
            &call,
            &quote!({}),
        )
    } else {
        call
    }
}

/// Generate the creation of the child through `factory` when the mount's
/// instance field is vacant.
///
/// The field is checked before the factory borrows the whole shared storage,
/// and a field that already holds an instance keeps it. Every token is
/// spanned at the value of the `factory` key, so a persistent field is
/// reported there.
fn factory_call(shared_storage: &Type, mount: &MountIr, factory: &Path) -> TokenStream {
    let MountIr { machine, field, .. } = mount;
    let shared_storage_parameter = generated_ident("shared_storage");
    let context = generated_ident("context");
    let instance = generated_ident("instance");
    let span = factory.span();
    let factory = factory_path(shared_storage, factory);

    quote_spanned! {span=>
        if ::sotu::private::FactoryInstance::<#machine>::is_vacant(&#shared_storage_parameter.#field) {
            let #instance = (#factory)(#shared_storage_parameter, #context);

            ::sotu::private::FactoryInstance::<#machine>::install(
                &mut #shared_storage_parameter.#field,
                #instance,
            );
        }
    }
}

/// Write `factory` so that it resolves outside the machine's impl block.
///
/// The generated call sits in an impl of a node enum, where `Self` names the
/// enum, so a leading `Self` segment is replaced by the shared storage type.
fn factory_path(shared_storage: &Type, factory: &Path) -> TokenStream {
    let mut segments = factory.segments.iter();

    match segments.next() {
        Some(first) if factory.leading_colon.is_none() && first.ident == "Self" => {
            let qualified = expression_path(shared_storage);
            let rest = segments.map(|segment| quote_spanned!(segment.span()=> ::#segment));

            quote_spanned! {first.span()=>
                #qualified #(#rest)*
            }
        }
        _ => factory.to_token_stream(),
    }
}

/// Generate the call of the child's entry or exit action `action` on the node
/// bound as `state`, with the child's storage and context.
///
/// The storage is absent only when user code emptied an active instance
/// field, and the action is then skipped.
fn child_action(mount: &MountIr, action: ChildCall, mode: Mode) -> TokenStream {
    let submachine = generated_ident("submachine");
    let storage = field_call(mount, "MountField", "get_mut");
    let action = child_call(&mount.machine, mode, action);
    let awaited = awaited(mode);

    let call = child_context_call(
        mount,
        |context| quote!(#action(state, #submachine, #context)#awaited),
    );

    quote! {
        if let ::core::option::Option::Some(#submachine) = #storage {
            #call;
        }
    }
}

/// Generate the callback into the child of `mount` that `call` builds from
/// the expression of the child's context.
///
/// Without a `context` mapping the parent's context is passed through. With
/// one, the mapping builds the child's context from the parent's for this
/// callback alone, and the child's context is dropped when the callback
/// returns, so its borrow of the parent's context ends before any parent hook
/// or later callback runs. The mapping is spanned at the value of the
/// `context` key, so a mapping of the wrong type is reported there.
fn child_context_call(
    mount: &MountIr,
    call: impl FnOnce(&TokenStream) -> TokenStream,
) -> TokenStream {
    let context = generated_ident("context");

    match &mount.context {
        Some(mapping) => {
            let child_context = generated_ident("child_context");
            let call = call(&quote!(&mut #child_context));

            quote_spanned! {mapping.span()=>
                {
                    let mut #child_context = (#mapping)(#context);

                    #call
                }
            }
        }
        None => call(&context.to_token_stream()),
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

// MARK: Mount Boundary

/// Generate the hidden associated functions of the shared storage that move a
/// parent event across each mount boundary, or nothing without mounts.
///
/// Each function checks `accepts`, projects the event into the child's event
/// type, hands it to `visit` and embeds the event that `visit` returns. A
/// parent event the child does not accept, or one whose projection fails, is
/// returned unchanged as `Err`. Keeping every use of `SubEvent` for a mount in
/// this one body reports a child event type that the parent does not list
/// exactly once, at the value of the `machine` key.
///
/// An awaitable parent addresses its child nodes synchronously, which a
/// closure that awaits the child can not do, so it generates a second function
/// per mount whose `visit` is an async closure, for the dispatch of the child
/// node.
fn projection_impl(machine: &MachineIr, mounts: &[MountIr], mode: Mode) -> Option<TokenStream> {
    if mounts.is_empty() {
        return None;
    }

    let shared_storage = &machine.shared_storage;
    let event_type = &machine.event_type;
    let (impl_generics, _, where_clause) = machine.generics.split_for_impl();
    let event = generated_ident("event");
    let visit = generated_ident("visit");

    let functions = mounts.iter().flat_map(|mount| {
        let MountIr {
            machine: child,
            projection,
            awaited_projection,
            ..
        } = mount;

        let child_event = quote!(<#child as ::sotu::private::Submachine>::Event);

        let sub_event = quote_spanned! {child.span()=>
            <<#child as ::sotu::private::Submachine>::Event as ::sotu::event::SubEvent<#event_type>>
        };

        // A child whose event enum has no variants has an uninhabited event
        // type, so the code after the visit is unreachable.
        let function = |name: &Ident, visit_mode: Mode| {
            let awaited = awaited(visit_mode);

            let (asyncness, visit_trait) = match visit_mode {
                Mode::Blocking => (None, quote!(::core::ops::FnOnce)),
                Mode::Awaitable => (Some(quote!(async)), quote!(::core::ops::AsyncFnOnce)),
            };

            quote! {
                #[doc(hidden)]
                #[allow(unreachable_code)]
                #asyncness fn #name<Visited>(
                    #event: #event_type,
                    #visit: impl #visit_trait(#child_event) -> (Visited, #child_event),
                ) -> ::core::result::Result<(Visited, #event_type), #event_type> {
                    if !#sub_event::accepts(&#event) {
                        return ::core::result::Result::Err(#event);
                    }

                    match #sub_event::project(#event) {
                        ::core::result::Result::Ok(child_event) => {
                            let (visited, child_event) = #visit(child_event)#awaited;

                            ::core::result::Result::Ok((visited, #sub_event::embed(child_event)))
                        }
                        ::core::result::Result::Err(#event) => ::core::result::Result::Err(#event),
                    }
                }
            }
        };

        let awaited_function =
            (mode == Mode::Awaitable).then(|| function(awaited_projection, Mode::Awaitable));

        [Some(function(projection, Mode::Blocking)), awaited_function]
    });

    let functions: Vec<TokenStream> = functions.flatten().collect();

    Some(quote! {
        impl #impl_generics #shared_storage #where_clause {
            #(#functions)*
        }
    })
}

/// Generate the expression that lifts `outcome`, a `NodeOutcome` of the child
/// of `mount`, into the parent's terms.
///
/// Targets are wrapped in the mount's state variant and offsets are kept,
/// because they count levels above the visited node, which is the same node in
/// both machines. An exit of the child goes to the mount's exit handler, and
/// without one the child is exitless, so its exit type is uninhabited.
fn lifted_outcome(
    state_name: &Ident,
    shared_storage: &Type,
    mount: &MountIr,
    outcome: &Ident,
) -> TokenStream {
    let variant = &mount.variant;

    let exit_arm = match &mount.exit_handler {
        Some(exit_handler) => {
            let exit = generated_ident("exit");
            let answer = exit_handler_answer(shared_storage, mount, exit_handler);

            quote! {
                ::sotu::private::NodeOutcome::Exit(#exit) => #answer,
            }
        }
        None => quote! {
            ::sotu::private::NodeOutcome::Exit(exit) => match exit {},
        },
    };

    quote! {
        match #outcome {
            ::sotu::private::NodeOutcome::Handled => ::sotu::private::NodeOutcome::Handled,
            ::sotu::private::NodeOutcome::Super => ::sotu::private::NodeOutcome::Super,
            ::sotu::private::NodeOutcome::Transition(target) => {
                ::sotu::private::NodeOutcome::Transition(#state_name::#variant(target))
            }
            ::sotu::private::NodeOutcome::ExternalTransition {
                target,
                handler_offset,
            } => ::sotu::private::NodeOutcome::ExternalTransition {
                target: #state_name::#variant(target),
                handler_offset,
            },
            #exit_arm
            ::sotu::private::NodeOutcome::DeferToMount { mount_offset } => {
                ::sotu::private::NodeOutcome::DeferToMount { mount_offset }
            }
        }
    }
}

/// Generate the call of `mount`'s exit handler with the child's exit value
/// and the expression that turns its answer into the parent's `NodeOutcome`.
///
/// The visited child node lies `mount_offset` levels below the mount node, so
/// an external transition is handled by the mount node, and `Super` continues
/// at the mount node, skipping the child nodes above the visited one. An exit
/// of the parent stays an exit and leaves through the next mount boundary or
/// terminates the root. The `submachine` input borrows the instance again
/// after the child's dispatch returned, and without the instance, which only
/// user code can empty, the exit handler is skipped as if it returned `Super`.
fn exit_handler_answer(
    shared_storage: &Type,
    mount: &MountIr,
    exit_handler: &MountActionIr,
) -> TokenStream {
    let handler = call(shared_storage, &exit_handler.call);
    let deferred = quote!(::sotu::private::NodeOutcome::DeferToMount { mount_offset });

    let answer = quote! {
        match #handler {
            ::sotu::Outcome::Handled => ::sotu::private::NodeOutcome::Handled,
            ::sotu::Outcome::Super => #deferred,
            ::sotu::Outcome::Transition(target) => {
                ::sotu::private::NodeOutcome::Transition(target)
            }
            ::sotu::Outcome::ExternalTransition(target) => {
                ::sotu::private::NodeOutcome::ExternalTransition {
                    target,
                    handler_offset: mount_offset,
                }
            }
            ::sotu::Outcome::Exit(parent_exit) => ::sotu::private::NodeOutcome::Exit(parent_exit),
        }
    };

    if exit_handler.takes_submachine {
        instance_call(
            mount,
            &generated_ident("shared_storage"),
            &answer,
            &deferred,
        )
    } else {
        answer
    }
}

/// The parts of a node enum's trait impl that the boundary methods read.
struct BoundaryContext<'ir> {
    /// The name of the node enum whose impl receives the methods.
    enum_name: &'ir Ident,
    /// The name of the state enum, whose mount variants wrap lifted targets.
    state_name: &'ir Ident,
    shared_storage: &'ir Type,
    event_type: &'ir Type,
    context_type: &'ir Type,
    exit_type: &'ir Type,
    /// The type `Dispatched` carries as the target of transitions.
    target_type: TokenStream,
    /// The trait whose `call_handler` handles the variants without a child.
    node_trait: TokenStream,
    /// The mode of the parent, which decides how it calls its children.
    mode: Mode,
    /// The calls into a child that report whether it is addressed, dispatch
    /// to it and measure its depth, for the kind of node that the methods
    /// visit.
    node_calls: NodeCalls,
}

/// The calls into a child with which a parent visits one kind of its nodes.
#[derive(Clone, Copy)]
struct NodeCalls {
    is_addressed: ChildCall,
    dispatch: ChildCall,
    depth: ChildCall,
}

/// Generate the arm of `__is_addressed` for a child node bound as `node`.
///
/// The child node is addressed exactly when the child accepts the event and
/// the child node itself reports it as addressed.
fn addressed_arm(
    context: &BoundaryContext<'_>,
    mount: &MountIr,
    pattern: &TokenStream,
) -> TokenStream {
    let storage = expression_path(context.shared_storage);
    let projection = &mount.projection;
    let event = generated_ident("event");

    let child_addressed = child_call(
        &mount.machine,
        context.mode,
        context.node_calls.is_addressed,
    );

    quote! {
        #pattern => match #storage::#projection(#event, |#event| {
            #child_addressed(node, #event)
        }) {
            ::core::result::Result::Ok(addressed) => addressed,
            ::core::result::Result::Err(#event) => (false, #event),
        },
    }
}

/// Generate the arm of `__dispatch` for a child node bound as `node`.
///
/// The child node is dispatched on the child's storage with the projected
/// event, and its outcome is lifted. The child's own dispatch hooks surround
/// the child node, seeing it as `child_node` and the event in the child's
/// types, so they nest inside the hooks of every enclosing machine. Without an
/// instance, or when the projection fails, the remaining child nodes are
/// skipped by deferring to the mount node, which sits as many levels above
/// `node` as `node` is deep inside the child.
fn dispatch_arm_into_child(
    context: &BoundaryContext<'_>,
    mount: &MountIr,
    pattern: &TokenStream,
    child_node: &TokenStream,
) -> TokenStream {
    let storage = expression_path(context.shared_storage);
    let machine = &mount.machine;
    let field = &mount.field;
    let mode = context.mode;
    let awaited = awaited(mode);
    let shared_storage = generated_ident("shared_storage");
    let event = generated_ident("event");
    let submachine = generated_ident("submachine");
    let outcome = generated_ident("outcome");
    let lifted = lifted_outcome(context.state_name, context.shared_storage, mount, &outcome);

    // An awaitable parent visits through an async closure, so that the visit
    // can await the child.
    let (projection, closure) = match mode {
        Mode::Blocking => (&mount.projection, quote!(|#event|)),
        Mode::Awaitable => (&mount.awaited_projection, quote!(async |#event|)),
    };

    let before_dispatch = child_call(machine, mode, ChildCall::BeforeDispatch);
    let after_dispatch = child_call(machine, mode, ChildCall::AfterDispatch);
    let child_dispatch = child_call(machine, mode, context.node_calls.dispatch);
    let child_depth = child_call(machine, mode, context.node_calls.depth);

    let before_dispatch = child_context_call(mount, |context| {
        quote! {
            #before_dispatch(
                #submachine,
                #child_node,
                &#event,
                #context,
            )#awaited
        }
    });

    let dispatch = child_context_call(
        mount,
        |context| quote!(#child_dispatch(node, #submachine, #event, #context)#awaited),
    );

    let after_dispatch = child_context_call(mount, |context| {
        quote! {
            #after_dispatch(
                #submachine,
                #child_node,
                &event,
                #context,
            )#awaited
        }
    });

    let storage_access = quote_spanned! {field.span()=>
        ::sotu::private::MountField::<#machine>::get_mut(&mut #shared_storage.#field)
    };

    quote! {
        #pattern => {
            let mount_offset = #child_depth(node);

            let ::core::option::Option::Some(#submachine) = #storage_access else {
                return ::sotu::private::Dispatched {
                    outcome: ::sotu::private::NodeOutcome::DeferToMount { mount_offset },
                    event: #event,
                };
            };

            let visited = #storage::#projection(#event, #closure {
                #before_dispatch;

                let ::sotu::private::Dispatched { outcome, event } = #dispatch;

                #after_dispatch;

                (outcome, event)
            })#awaited;

            match visited {
                ::core::result::Result::Ok((#outcome, #event)) => ::sotu::private::Dispatched {
                    outcome: #lifted,
                    event: #event,
                },
                ::core::result::Result::Err(#event) => ::sotu::private::Dispatched {
                    outcome: ::sotu::private::NodeOutcome::DeferToMount { mount_offset },
                    event: #event,
                },
            }
        }
    }
}

/// Generate `__is_addressed` and `__dispatch` of a node enum whose child
/// nodes are matched by `child_arms`, pairs of an `__is_addressed` arm and a
/// `__dispatch` arm.
///
/// Every other variant is addressed and dispatched as the trait's defaults
/// do. `has_other_variants` tells whether such a variant exists, since a
/// wildcard arm without one would be reported as unreachable in the user's
/// crate.
fn boundary_methods(
    context: &BoundaryContext<'_>,
    child_arms: Vec<(TokenStream, TokenStream)>,
    has_other_variants: bool,
) -> TokenStream {
    let BoundaryContext {
        shared_storage,
        event_type,
        context_type,
        exit_type,
        target_type,
        node_trait,
        mode,
        ..
    } = context;

    let shared_storage_parameter = generated_ident("shared_storage");
    let event = generated_ident("event");
    let ctx = generated_ident("context");
    let (addressed_arms, dispatch_arms): (Vec<_>, Vec<_>) = child_arms.into_iter().unzip();
    let awaited = awaited(*mode);

    let other_addressed = has_other_variants.then(|| quote!(_ => (true, #event),));

    let other_dispatched = has_other_variants.then(|| {
        quote! {
            _ => {
                let outcome = #node_trait::call_handler(
                    self,
                    #shared_storage_parameter,
                    &#event,
                    #ctx,
                )#awaited;

                ::sotu::private::Dispatched {
                    outcome: ::sotu::private::NodeOutcome::from(outcome),
                    event: #event,
                }
            }
        }
    });

    let dispatched = quote!(::sotu::private::Dispatched<#target_type, #exit_type, #event_type>);
    let output = method_output(*mode, Some(&dispatched));

    let body = method_body(
        *mode,
        quote! {
            match self {
                #(#dispatch_arms)*
                #other_dispatched
            }
        },
    );

    // A child whose event enum has no variants has an uninhabited event type,
    // so the code after its dispatch is unreachable.
    quote! {
        fn __is_addressed(&self, #event: #event_type) -> (bool, #event_type) {
            match self {
                #(#addressed_arms)*
                #other_addressed
            }
        }

        #[allow(unreachable_code)]
        fn __dispatch<'ctx, 'data: 'ctx>(
            &mut self,
            #shared_storage_parameter: &mut #shared_storage,
            #event: #event_type,
            #ctx: &mut #context_type,
        ) #output {
            #body
        }
    }
}

/// Generate `__is_addressed` and `__dispatch` of the state enum, which visit
/// the child's leaf state inside each mount variant.
fn state_boundary_methods(dispatch: &DispatchIr, mounts: &[MountIr], mode: Mode) -> TokenStream {
    let name = &dispatch.state.name;
    let shared_storage = &dispatch.shared_storage;
    let module = mode_module(mode);

    let context = BoundaryContext {
        enum_name: name,
        state_name: name,
        shared_storage,
        event_type: &dispatch.event_type,
        context_type: &dispatch.context_type,
        exit_type: &dispatch.exit_type,
        target_type: quote!(Self),
        node_trait: quote!(#module::State::<#shared_storage>),
        mode,
        node_calls: NodeCalls {
            is_addressed: ChildCall::StateIsAddressed,
            dispatch: ChildCall::StateDispatch,
            depth: ChildCall::StateDepth,
        },
    };

    let child_arms = mounts
        .iter()
        .map(|mount| {
            let enum_name = context.enum_name;
            let variant = &mount.variant;
            let pattern = quote!(#enum_name::#variant(node));

            (
                addressed_arm(&context, mount, &pattern),
                dispatch_arm_into_child(
                    &context,
                    mount,
                    &pattern,
                    &quote!(::sotu::StateOrSuperstate::State(&*node)),
                ),
            )
        })
        .collect();

    boundary_methods(&context, child_arms, !dispatch.arms.is_empty())
}

/// Generate `__is_addressed` and `__dispatch` of the superstate enum, which
/// visit the child's superstate inside each mount variant. The mount node
/// itself is handled by the mount handler like any other superstate.
fn superstate_boundary_methods(
    superstate: &SuperstateIr,
    mounts: &[MountIr],
    mode: Mode,
) -> TokenStream {
    let shared_storage = &superstate.shared_storage;
    let module = mode_module(mode);

    let context = BoundaryContext {
        enum_name: &superstate.ty.name,
        state_name: &superstate.state.name,
        shared_storage,
        event_type: &superstate.event_type,
        context_type: &superstate.context_type,
        exit_type: &superstate.exit_type,
        target_type: state_type(&superstate.state),
        node_trait: quote!(#module::Superstate::<#shared_storage>),
        mode,
        node_calls: NodeCalls {
            is_addressed: ChildCall::SuperstateIsAddressed,
            dispatch: ChildCall::SuperstateDispatch,
            depth: ChildCall::SuperstateDepth,
        },
    };

    let child_arms = mounts
        .iter()
        .map(|mount| {
            let enum_name = context.enum_name;
            let variant = &mount.variant;
            let pattern = quote!(#enum_name::#variant(::sotu::SubmachineNode::Inner(node)));

            (
                addressed_arm(&context, mount, &pattern),
                dispatch_arm_into_child(
                    &context,
                    mount,
                    &pattern,
                    &quote!(::sotu::StateOrSuperstate::Superstate(&*node)),
                ),
            )
        })
        .collect();

    // The mount node of every mount is a variant without a child node.
    boundary_methods(&context, child_arms, true)
}

/// Generate `__internal_transition_nesting`, `__before_internal_transition`
/// and `__after_internal_transition` of the state enum, which run the
/// transition hooks of every child instance that a transition stays inside.
///
/// A transition from `Variant(source)` retains the mount node of `Variant`
/// exactly when it exits no more levels than `source` lies deep inside the
/// child, so the mount node is a common ancestor of source and target that is
/// neither exited nor entered, and both states belong to the same child
/// instance. Since every machine counts the exited levels from the same active
/// leaf, the same `exit_levels` decides retention inside the child as well.
///
/// The nesting of a transition is the number of nested instances it stays
/// inside. Level `1` runs the hooks of this machine's own child, and each
/// deeper level is handed to the child with one level less. Without the
/// child's storage, which only user code can empty, the hooks are skipped.
fn internal_transition_methods(
    dispatch: &DispatchIr,
    mounts: &[MountIr],
    mode: Mode,
) -> TokenStream {
    let name = &dispatch.state.name;
    let shared_storage_type = &dispatch.shared_storage;
    let context_type = &dispatch.context_type;
    let target = generated_ident("target");
    let exit_levels = generated_ident("exit_levels");
    let nesting = generated_ident("nesting");
    let shared_storage = generated_ident("shared_storage");
    let context = generated_ident("context");
    let submachine = generated_ident("submachine");
    let awaited = awaited(mode);

    // With a single variant the pairs of equal variants are exhaustive, and a
    // wildcard arm would be reported as unreachable in the user's crate.
    let has_other_pairs = dispatch.arms.len() + mounts.len() > 1;

    let nesting_arms = mounts.iter().map(|mount| {
        let variant = &mount.variant;
        let child_depth = child_call(&mount.machine, mode, ChildCall::StateDepth);

        let child_nesting = child_call(&mount.machine, mode, ChildCall::InternalTransitionNesting);

        quote! {
            (#name::#variant(source), #name::#variant(#target)) => {
                if #exit_levels <= #child_depth(source) {
                    1 + #child_nesting(source, #target, #exit_levels)
                } else {
                    0
                }
            }
        }
    });

    let other_nesting = has_other_pairs.then(|| quote!(_ => 0,));

    let [before, after] = [
        (
            "__before_internal_transition",
            ChildCall::BeforeInternalTransition,
            ChildCall::BeforeTransition,
        ),
        (
            "__after_internal_transition",
            ChildCall::AfterInternalTransition,
            ChildCall::AfterTransition,
        ),
    ]
    .map(|(method, deeper_function, hook_function)| {
        let method = format_ident!("{method}");

        let arms = mounts.iter().map(|mount| {
            let variant = &mount.variant;
            let machine = &mount.machine;
            let child_depth = child_call(machine, mode, ChildCall::StateDepth);
            let child_deeper = child_call(machine, mode, deeper_function);
            let child_hook = child_call(machine, mode, hook_function);
            let storage = field_call(mount, "MountField", "get_mut");

            let hook_call = child_context_call(mount, |context| {
                quote! {
                    #child_hook(
                        #submachine,
                        source,
                        #target,
                        #context,
                    )#awaited
                }
            });

            let deeper_call = child_context_call(mount, |context| {
                quote! {
                    #child_deeper(
                        source,
                        #target,
                        #exit_levels,
                        deeper,
                        #submachine,
                        #context,
                    )#awaited
                }
            });

            quote! {
                (#name::#variant(source), #name::#variant(#target)) => {
                    if #exit_levels > #child_depth(source) {
                        return;
                    }

                    let ::core::option::Option::Some(#submachine) = #storage else {
                        return;
                    };

                    match #nesting.checked_sub(1) {
                        ::core::option::Option::Some(0) => {
                            #hook_call;
                        }
                        ::core::option::Option::Some(deeper) => {
                            #deeper_call;
                        }
                        ::core::option::Option::None => {}
                    }
                }
            }
        });

        let other = has_other_pairs.then(|| quote!(_ => {}));
        let output = method_output(mode, None);

        let body = method_body(
            mode,
            quote! {
                match (self, #target) {
                    #(#arms)*
                    #other
                }
            },
        );

        quote! {
            fn #method<'ctx, 'data: 'ctx>(
                &mut self,
                #target: &mut Self,
                #exit_levels: usize,
                #nesting: usize,
                #shared_storage: &mut #shared_storage_type,
                #context: &mut #context_type,
            ) #output {
                #body
            }
        }
    });

    quote! {
        fn __internal_transition_nesting(&mut self, #target: &mut Self, #exit_levels: usize) -> usize {
            match (self, #target) {
                #(#nesting_arms)*
                #other_nesting
            }
        }

        #before

        #after
    }
}
