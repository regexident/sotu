//! Depth and transition path computations over a state hierarchy.
//!
//! `local_path` and `external_path` hold the path arithmetic, which is the
//! same in every mode. The walks along the chain of superstates above a leaf
//! state differ between modes, so a macro generates them in the module that
//! invokes it, where the names `IntoStateMachine`, `State` and `Superstate`
//! refer to that mode's traits.
//!
//! `hierarchy_algorithms!` generates walks for a `superstate` method that
//! borrows the superstate mutably. Every recursive step borrows the next
//! superstate from the previous one, so at most one borrow of each chain is
//! live at a time.
//!
//! `owned_hierarchy_algorithms!` generates walks for a `superstate` method
//! that consumes the superstate and returns its parent. Every step replaces
//! the current superstate with its parent, so the walks are loops and keep at
//! most one superstate of each chain alive at a time.
//!
//! Depths count levels from the top: a top-level node has depth 1 and the
//! implicit top above all nodes has depth 0.
//!
//! A local transition from the leaf `L` to a target exits the levels below the
//! deepest common ancestor of the two and enters the levels below it again. A
//! transition to the same node exits and enters exactly that node. An external
//! transition handled by the node `H` additionally exits and re-enters `H`
//! when `H` is the source or an ancestor of both source and target.
//!
//! The deepest common ancestor is found by climbing the deeper chain until
//! both are equally deep and then climbing both in lockstep, comparing nodes
//! with `same_state`. Each chain is walked once, so the cost is linear in the
//! depth of the hierarchy.

// MARK: Path Arithmetic

/// Return how many levels a local transition exits and enters, as
/// `(exit_levels, enter_levels)`.
///
/// `source_depth` and `target_depth` are the depths of the source and target
/// leaves, and `common_depth` is the depth of their deepest common ancestor,
/// or `0` if they have none. When `same_leaf` is `true`, the transition exits
/// and enters exactly the source leaf and the depths are not consulted.
pub(crate) fn local_path(
    source_depth: usize,
    target_depth: usize,
    common_depth: usize,
    same_leaf: bool,
) -> (usize, usize) {
    if same_leaf {
        return (1, 1);
    }

    (source_depth - common_depth, target_depth - common_depth)
}

/// Return how many levels an external transition exits and enters, as
/// `(exit_levels, enter_levels)`.
///
/// The transition is handled by the node at `handler_depth` on the chain of
/// the source leaf, which has depth `source_depth`. A `handler_depth` of `0`
/// or beyond `source_depth` names no handling node. `target_inside_handler`
/// reports whether the target leaf, at `target_depth`, is the handling node or
/// lies below it. In that case the handling node and every level below it are
/// exited and entered again. Otherwise the transition takes the levels of the
/// local transition between the same leaves, which `local` holds.
pub(crate) fn external_path(
    source_depth: usize,
    target_depth: usize,
    handler_depth: usize,
    target_inside_handler: bool,
    local: (usize, usize),
) -> (usize, usize) {
    let names_handler = handler_depth != 0 && handler_depth <= source_depth;

    if names_handler && target_inside_handler {
        return (
            source_depth - handler_depth + 1,
            target_depth - handler_depth + 1,
        );
    }

    local
}

// MARK: Borrowing Walks

/// Generate the hierarchy algorithms in the invoking module.
///
/// The invoking module must have `IntoStateMachine`, `State` and `Superstate`
/// in scope, naming the traits of one mode.
macro_rules! hierarchy_algorithms {
    () => {
        /// Return the depth of `state`, counting itself.
        pub(crate) fn state_depth<M: IntoStateMachine>(state: &mut M::State) -> usize {
            match state.superstate() {
                Some(mut superstate) => 1 + superstate_depth::<M>(&mut superstate),
                None => 1,
            }
        }

        /// Return the depth of `superstate`, counting itself.
        pub(crate) fn superstate_depth<M: IntoStateMachine>(
            superstate: &mut M::Superstate<'_>,
        ) -> usize {
            match superstate.superstate() {
                Some(mut parent) => 1 + superstate_depth::<M>(&mut parent),
                None => 1,
            }
        }

        /// Return how many levels a local transition from `source` to `target`
        /// exits and enters, as `(exit_levels, enter_levels)`.
        pub(crate) fn transition_path<M: IntoStateMachine>(
            source: &mut M::State,
            target: &mut M::State,
        ) -> (usize, usize) {
            let same_leaf = source.same_state(target);
            let source_depth = state_depth::<M>(source);
            let target_depth = state_depth::<M>(target);

            // A leaf of depth 1 has no superstate, so its only common ancestor
            // with any other leaf is the implicit top at depth 0.
            let common_depth = match (source.superstate(), target.superstate()) {
                (Some(mut source_superstate), Some(mut target_superstate)) => {
                    common_ancestor_depth::<M>(
                        &mut source_superstate,
                        source_depth - 1,
                        &mut target_superstate,
                        target_depth - 1,
                    )
                }
                _ => 0,
            };

            crate::hierarchy::local_path(source_depth, target_depth, common_depth, same_leaf)
        }

        /// Return how many levels an external transition from `source` to
        /// `target`, handled by the node at `handler_depth`, exits and enters,
        /// as `(exit_levels, enter_levels)`.
        ///
        /// A `handler_depth` of `0` or beyond the depth of `source` names no
        /// handling node, so the transition is treated as a local one.
        pub(crate) fn external_transition_path<M: IntoStateMachine>(
            source: &mut M::State,
            target: &mut M::State,
            handler_depth: usize,
        ) -> (usize, usize) {
            let source_depth = state_depth::<M>(source);
            let target_depth = state_depth::<M>(target);

            let target_inside_handler = if handler_depth == source_depth {
                source.same_state(target)
            } else if (1..source_depth).contains(&handler_depth) && target_depth > handler_depth {
                // Both depths exceed `handler_depth`, so both leaves have a
                // superstate.
                match (source.superstate(), target.superstate()) {
                    (Some(mut source_superstate), Some(mut target_superstate)) => {
                        same_ancestor_at_depth::<M>(
                            &mut source_superstate,
                            source_depth - 1,
                            &mut target_superstate,
                            target_depth - 1,
                            handler_depth,
                        )
                    }
                    _ => false,
                }
            } else {
                false
            };

            let local = transition_path::<M>(source, target);

            crate::hierarchy::external_path(
                source_depth,
                target_depth,
                handler_depth,
                target_inside_handler,
                local,
            )
        }

        /// Return the depth of the deepest superstate that is an ancestor of,
        /// or the same node as, both `first` and `second`, or `0` if there is
        /// none.
        ///
        /// `first_depth` and `second_depth` are the depths of the two
        /// superstates.
        fn common_ancestor_depth<M: IntoStateMachine>(
            first: &mut M::Superstate<'_>,
            first_depth: usize,
            second: &mut M::Superstate<'_>,
            second_depth: usize,
        ) -> usize {
            if first_depth > second_depth {
                return match first.superstate() {
                    Some(mut parent) => common_ancestor_depth::<M>(
                        &mut parent,
                        first_depth - 1,
                        second,
                        second_depth,
                    ),
                    None => 0,
                };
            }

            if second_depth > first_depth {
                return match second.superstate() {
                    Some(mut parent) => common_ancestor_depth::<M>(
                        first,
                        first_depth,
                        &mut parent,
                        second_depth - 1,
                    ),
                    None => 0,
                };
            }

            if first.same_state(second) {
                return first_depth;
            }

            match (first.superstate(), second.superstate()) {
                (Some(mut first_parent), Some(mut second_parent)) => common_ancestor_depth::<M>(
                    &mut first_parent,
                    first_depth - 1,
                    &mut second_parent,
                    second_depth - 1,
                ),
                _ => 0,
            }
        }

        /// Return whether the ancestors of `first` and `second` at `depth` are
        /// the same node.
        ///
        /// `first_depth` and `second_depth` are the depths of the two
        /// superstates and must not be smaller than `depth`.
        fn same_ancestor_at_depth<M: IntoStateMachine>(
            first: &mut M::Superstate<'_>,
            first_depth: usize,
            second: &mut M::Superstate<'_>,
            second_depth: usize,
            depth: usize,
        ) -> bool {
            if first_depth > depth {
                return match first.superstate() {
                    Some(mut parent) => same_ancestor_at_depth::<M>(
                        &mut parent,
                        first_depth - 1,
                        second,
                        second_depth,
                        depth,
                    ),
                    None => false,
                };
            }

            if second_depth > depth {
                return match second.superstate() {
                    Some(mut parent) => same_ancestor_at_depth::<M>(
                        first,
                        first_depth,
                        &mut parent,
                        second_depth - 1,
                        depth,
                    ),
                    None => false,
                };
            }

            first.same_state(second)
        }
    };
}

pub(crate) use hierarchy_algorithms;

// MARK: Owned Walks

/// Generate the hierarchy algorithms in the invoking module for a mode whose
/// `Superstate::superstate` consumes the superstate.
///
/// The invoking module must have `IntoStateMachine`, `State` and `Superstate`
/// in scope, naming the traits of one mode.
#[cfg(feature = "async")]
macro_rules! owned_hierarchy_algorithms {
    () => {
        /// Return the depth of `state`, counting itself.
        pub(crate) fn state_depth<M: IntoStateMachine>(state: &mut M::State) -> usize {
            match state.superstate() {
                Some(superstate) => 1 + superstate_depth::<M>(superstate),
                None => 1,
            }
        }

        /// Return the depth of `superstate`, counting itself.
        pub(crate) fn superstate_depth<M: IntoStateMachine>(
            superstate: M::Superstate<'_>,
        ) -> usize {
            let mut depth = 1;
            let mut current = superstate.superstate();

            while let Some(parent) = current {
                depth += 1;
                current = parent.superstate();
            }

            depth
        }
    };
}

#[cfg(feature = "async")]
pub(crate) use owned_hierarchy_algorithms;
