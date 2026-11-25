#![cfg(feature = "async")]

mod common;

use sotu::private::AwaitableSubmachine;

// MARK: Blocking Machine

mod nested {
    use sotu::prelude::*;

    #[derive(Debug, PartialEq, Eq)]
    pub struct Bump;

    #[derive(Event, Debug, PartialEq, Eq)]
    pub enum Event {
        Bump(Bump),
    }

    #[derive(Default)]
    pub struct Nested {
        pub trace: Vec<&'static str>,
    }

    #[state_machine(
        initial = State::on(0),
        state(derive(Debug, PartialEq, Eq)),
        superstate(derive(Debug, PartialEq, Eq)),
        before_dispatch = Self::record_before_dispatch,
        after_dispatch = Self::record_after_dispatch,
        before_transition = Self::record_before_transition,
        after_transition = Self::record_after_transition
    )]
    impl Nested {
        fn record_before_dispatch(
            &mut self,
            _node: StateOrSuperstate<'_, State, Superstate<'_>>,
            _event: &Event,
        ) {
            self.trace.push("before dispatch");
        }

        fn record_after_dispatch(
            &mut self,
            _node: StateOrSuperstate<'_, State, Superstate<'_>>,
            _event: &Event,
        ) {
            self.trace.push("after dispatch");
        }

        fn record_before_transition(&mut self, _source: &State, _target: &State) {
            self.trace.push("before transition");
        }

        fn record_after_transition(&mut self, _source: &State, _target: &State) {
            self.trace.push("after transition");
        }

        #[state(
            superstate = inner,
            entry_action = enter_on,
            exit_action = exit_on
        )]
        fn on(counter: &mut u32, event: &Event) -> Outcome<State> {
            match event {
                Event::Bump(_) => {
                    *counter += 1;

                    Transition(State::off())
                }
            }
        }

        #[state]
        fn off(event: &Event) -> Outcome<State> {
            match event {
                Event::Bump(_) => Handled,
            }
        }

        #[superstate(
            superstate = outer,
            entry_action = enter_inner,
            exit_action = exit_inner
        )]
        fn inner(counter: &mut u32, event: &Event) -> Outcome<State> {
            match event {
                Event::Bump(_) => {
                    *counter += 10;

                    Super
                }
            }
        }

        #[superstate]
        fn outer(counter: &u32, event: &Event) -> Outcome<State> {
            match event {
                Event::Bump(_) => {
                    if *counter == 0 {
                        Handled
                    } else {
                        Transition(State::off())
                    }
                }
            }
        }

        #[action]
        fn enter_on(&mut self) {
            self.trace.push("enter on");
        }

        #[action]
        fn exit_on(&mut self) {
            self.trace.push("exit on");
        }

        #[action]
        fn enter_inner(&mut self) {
            self.trace.push("enter inner");
        }

        #[action]
        fn exit_inner(&mut self) {
            self.trace.push("exit inner");
        }
    }
}

#[test]
fn blocking_machine_bridges_hierarchy_queries() {
    use nested::{Nested, State, Superstate};

    let mut state = State::on(7);

    assert_eq!(<Nested as AwaitableSubmachine>::state_depth(&mut state), 3);
    assert_eq!(
        <Nested as AwaitableSubmachine>::state_depth(&mut State::off()),
        1
    );

    let Some(inner) = <Nested as AwaitableSubmachine>::state_superstate(&mut state) else {
        panic!("`on` has no superstate");
    };

    assert_eq!(<Nested as AwaitableSubmachine>::superstate_depth(&inner), 2);

    // The parent takes the borrowed storage over from its child.
    let Some(outer) = <Nested as AwaitableSubmachine>::superstate_parent(inner) else {
        panic!("`inner` has no parent");
    };

    assert_eq!(<Nested as AwaitableSubmachine>::superstate_depth(&outer), 1);
    assert!(matches!(&outer, Superstate::Outer { counter } if **counter == 7));
    assert!(<Nested as AwaitableSubmachine>::superstate_parent(outer).is_none());

    assert!(<Nested as AwaitableSubmachine>::state_superstate(&mut State::off()).is_none());

    let mut first = State::on(1);
    let mut second = State::on(2);

    assert!(<Nested as AwaitableSubmachine>::state_same(&first, &second));
    assert!(!<Nested as AwaitableSubmachine>::state_same(
        &first,
        &State::off()
    ));

    let first_inner = <Nested as AwaitableSubmachine>::state_superstate(&mut first);
    let second_inner = <Nested as AwaitableSubmachine>::state_superstate(&mut second);

    let (Some(first_inner), Some(second_inner)) = (first_inner, second_inner) else {
        panic!("`on` has no superstate");
    };

    assert!(<Nested as AwaitableSubmachine>::superstate_same(
        &first_inner,
        &second_inner
    ));

    let Some(second_outer) = <Nested as AwaitableSubmachine>::superstate_parent(second_inner)
    else {
        panic!("`inner` has no parent");
    };

    assert!(!<Nested as AwaitableSubmachine>::superstate_same(
        &first_inner,
        &second_outer
    ));
}

#[test]
fn blocking_machine_bridges_dispatch_as_ready_future() {
    use futures::FutureExt as _;
    use nested::{Bump, Event, Nested, State};
    use sotu::StateOrSuperstate;
    use sotu::private::{Dispatched, NodeOutcome};

    let mut storage = Nested::default();
    let mut state = State::on(0);

    let mut reference_storage = Nested::default();
    let mut reference_state = State::on(0);

    let expected = <State as sotu::blocking::State<Nested>>::__dispatch(
        &mut reference_state,
        &mut reference_storage,
        Event::Bump(Bump),
        &mut (),
    );

    assert_eq!(
        expected,
        Dispatched {
            outcome: NodeOutcome::Transition(State::off()),
            event: Event::Bump(Bump),
        }
    );

    // A blocking machine runs to completion when the future is created, so
    // the future is ready on its first poll.
    let dispatched = <Nested as AwaitableSubmachine>::dispatch_state(
        &mut state,
        &mut storage,
        Event::Bump(Bump),
        &mut (),
    )
    .now_or_never();

    assert_eq!(dispatched, Some(expected));
    assert_eq!(state, reference_state);
    assert_eq!(state, State::on(1));
    assert_eq!(
        <Nested as AwaitableSubmachine>::state_is_addressed(&state, Event::Bump(Bump)),
        (true, Event::Bump(Bump))
    );

    // The superstates dispatch to their own handlers and defer upwards.
    let Some(mut inner) = <Nested as AwaitableSubmachine>::state_superstate(&mut state) else {
        panic!("`on` has no superstate");
    };

    let dispatched = <Nested as AwaitableSubmachine>::dispatch_superstate(
        &mut inner,
        &mut storage,
        Event::Bump(Bump),
        &mut (),
    )
    .now_or_never();

    assert_eq!(
        dispatched,
        Some(Dispatched {
            outcome: NodeOutcome::Super,
            event: Event::Bump(Bump),
        })
    );

    let Some(mut outer) = <Nested as AwaitableSubmachine>::superstate_parent(inner) else {
        panic!("`inner` has no parent");
    };

    let dispatched = <Nested as AwaitableSubmachine>::dispatch_superstate(
        &mut outer,
        &mut storage,
        Event::Bump(Bump),
        &mut (),
    )
    .now_or_never();

    assert_eq!(
        dispatched,
        Some(Dispatched {
            outcome: NodeOutcome::Transition(State::off()),
            event: Event::Bump(Bump),
        })
    );

    assert_eq!(
        <Nested as AwaitableSubmachine>::superstate_is_addressed(&outer, Event::Bump(Bump)),
        (true, Event::Bump(Bump))
    );

    // Actions and hooks run on the shared storage when their futures are
    // created, in the order the boundary is called.
    let mut storage = Nested::default();
    let mut state = State::on(0);
    let event = Event::Bump(Bump);

    assert_eq!(
        <Nested as AwaitableSubmachine>::enter_state(&mut state, &mut storage, &mut ())
            .now_or_never(),
        Some(())
    );
    assert_eq!(
        <Nested as AwaitableSubmachine>::exit_state(&mut state, &mut storage, &mut ())
            .now_or_never(),
        Some(())
    );

    let Some(mut inner) = <Nested as AwaitableSubmachine>::state_superstate(&mut state) else {
        panic!("`on` has no superstate");
    };

    assert_eq!(
        <Nested as AwaitableSubmachine>::enter_superstate(&mut inner, &mut storage, &mut ())
            .now_or_never(),
        Some(())
    );
    assert_eq!(
        <Nested as AwaitableSubmachine>::exit_superstate(&mut inner, &mut storage, &mut ())
            .now_or_never(),
        Some(())
    );

    assert_eq!(
        <Nested as AwaitableSubmachine>::before_dispatch(
            &mut storage,
            StateOrSuperstate::Superstate(&inner),
            &event,
            &mut ()
        )
        .now_or_never(),
        Some(())
    );
    assert_eq!(
        <Nested as AwaitableSubmachine>::after_dispatch(
            &mut storage,
            StateOrSuperstate::Superstate(&inner),
            &event,
            &mut ()
        )
        .now_or_never(),
        Some(())
    );

    assert_eq!(
        <Nested as AwaitableSubmachine>::before_transition(
            &mut storage,
            &State::on(0),
            &State::off(),
            &mut ()
        )
        .now_or_never(),
        Some(())
    );
    assert_eq!(
        <Nested as AwaitableSubmachine>::after_transition(
            &mut storage,
            &State::on(0),
            &State::off(),
            &mut ()
        )
        .now_or_never(),
        Some(())
    );

    assert_eq!(
        storage.trace,
        [
            "enter on",
            "exit on",
            "enter inner",
            "exit inner",
            "before dispatch",
            "after dispatch",
            "before transition",
            "after transition",
        ]
    );

    // A machine without mounts has no instances to run transition hooks of.
    let mut storage = Nested::default();
    let mut source = State::on(0);
    let mut target = State::off();

    assert_eq!(
        <Nested as AwaitableSubmachine>::internal_transition_nesting(&mut source, &mut target, 3),
        0
    );
    assert_eq!(
        <Nested as AwaitableSubmachine>::before_internal_transition(
            &mut source,
            &mut target,
            3,
            0,
            &mut storage,
            &mut ()
        )
        .now_or_never(),
        Some(())
    );
    assert_eq!(
        <Nested as AwaitableSubmachine>::after_internal_transition(
            &mut source,
            &mut target,
            3,
            0,
            &mut storage,
            &mut ()
        )
        .now_or_never(),
        Some(())
    );
    assert!(storage.trace.is_empty());
}
