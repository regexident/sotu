use crate::StateOrSuperstate;
use crate::private::{WithContext, WithoutContext};

use super::IntoStateMachine;

// MARK: Dispatch Hooks

/// Call a dispatch hook function, with or without a context parameter, and
/// return a future that completes when the hook has run.
///
/// Implemented for every function that takes the shared storage, the visited
/// node and the event, optionally followed by the context. Code generated for
/// a machine cannot tell from a hook's path what shape the hook has, so it
/// awaits [`call_dispatch_hook`](DispatchHook::call_dispatch_hook) and lets
/// the compiler pick the impl. A hook without a context parameter never sees
/// the context.
///
/// For a plain function returning `()`, the `Arity` marker is
/// [`WithoutContext`] or [`WithContext`]. Such a function matches exactly one
/// of the two, so the marker is always inferred. A plain hook runs to
/// completion before the call returns, and the returned future is ready.
pub trait DispatchHook<'ctx, 'data: 'ctx, M: IntoStateMachine, Arity> {
    /// Call the hook for `node` and `event`, forwarding `context` if the hook
    /// takes it.
    fn call_dispatch_hook(
        &self,
        shared_storage: &mut M,
        node: StateOrSuperstate<'_, M::State, M::Superstate<'_>>,
        event: &M::Event,
        context: &mut M::Context<'ctx, 'data>,
    ) -> impl Future<Output = ()>;
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
    ) -> impl Future<Output = ()> {
        self(shared_storage, node, event);

        core::future::ready(())
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
    ) -> impl Future<Output = ()> {
        self(shared_storage, node, event, context);

        core::future::ready(())
    }
}
