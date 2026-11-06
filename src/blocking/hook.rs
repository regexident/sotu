use crate::StateOrSuperstate;
use crate::private::{WithContext, WithoutContext};

use super::IntoStateMachine;

/// Call a dispatch hook function, with or without a context parameter.
///
/// Implemented for every function that takes the shared storage, the visited
/// node and the event, optionally followed by the context. Code generated for
/// a machine cannot tell from a hook's path whether the hook takes the context,
/// so it calls [`call_dispatch_hook`](DispatchHook::call_dispatch_hook) and
/// lets the compiler pick the impl.
///
/// The `Arity` marker is [`WithoutContext`] or [`WithContext`]. A function
/// matches exactly one of the two, so the marker is always inferred. A hook
/// without a context parameter never sees the context.
pub trait DispatchHook<'ctx, 'data: 'ctx, M: IntoStateMachine, Arity> {
    /// Call the hook for `node` and `event`, forwarding `context` if the hook
    /// takes it.
    fn call_dispatch_hook(
        &self,
        shared_storage: &mut M,
        node: StateOrSuperstate<'_, M::State, M::Superstate<'_>>,
        event: &M::Event,
        context: &mut M::Context<'ctx, 'data>,
    );
}

impl<'ctx, 'data: 'ctx, M, F> DispatchHook<'ctx, 'data, M, WithoutContext> for F
where
    M: IntoStateMachine,
    F: Fn(&mut M, StateOrSuperstate<'_, M::State, M::Superstate<'_>>, &M::Event),
{
    fn call_dispatch_hook(
        &self,
        shared_storage: &mut M,
        node: StateOrSuperstate<'_, M::State, M::Superstate<'_>>,
        event: &M::Event,
        _context: &mut M::Context<'ctx, 'data>,
    ) {
        self(shared_storage, node, event);
    }
}

impl<'ctx, 'data: 'ctx, M, F> DispatchHook<'ctx, 'data, M, WithContext> for F
where
    M: IntoStateMachine,
    F: Fn(
        &mut M,
        StateOrSuperstate<'_, M::State, M::Superstate<'_>>,
        &M::Event,
        &mut M::Context<'ctx, 'data>,
    ),
{
    fn call_dispatch_hook(
        &self,
        shared_storage: &mut M,
        node: StateOrSuperstate<'_, M::State, M::Superstate<'_>>,
        event: &M::Event,
        context: &mut M::Context<'ctx, 'data>,
    ) {
        self(shared_storage, node, event, context);
    }
}
