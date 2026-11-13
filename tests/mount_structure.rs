mod common;

use common::trace;
use sotu::blocking::StateExt;
use sotu::prelude::*;

mod door {
    use sotu::prelude::*;

    use super::trace;

    pub struct Door {
        pub openings: u32,
    }

    impl Default for Door {
        fn default() -> Self {
            trace::record("door created");

            Self { openings: 0 }
        }
    }

    impl Drop for Door {
        fn drop(&mut self) {
            trace::record("door dropped");
        }
    }

    #[derive(Event)]
    pub enum Event {}

    #[state_machine(initial = State::opened())]
    impl Door {
        #[state(superstate = frame, entry_action = enter_opened, exit_action = exit_opened)]
        fn opened(event: &Event) -> Outcome<State> {
            match *event {}
        }

        #[state(superstate = frame, entry_action = enter_closed, exit_action = exit_closed)]
        fn closed(event: &Event) -> Outcome<State> {
            match *event {}
        }

        #[superstate(entry_action = enter_frame, exit_action = exit_frame)]
        fn frame(event: &Event) -> Outcome<State> {
            match *event {}
        }

        #[action]
        fn enter_opened(&mut self) {
            self.openings += 1;
            trace::record("enter opened");
        }

        #[action]
        fn exit_opened() {
            trace::record("exit opened");
        }

        #[action]
        fn enter_closed() {
            trace::record("enter closed");
        }

        #[action]
        fn exit_closed() {
            trace::record("exit closed");
        }

        #[action]
        fn enter_frame() {
            trace::record("enter frame");
        }

        #[action]
        fn exit_frame() {
            trace::record("exit frame");
        }
    }
}

mod hall {
    use sotu::prelude::*;

    use super::door;

    #[derive(Default)]
    pub struct Hall {
        pub door: Option<door::Door>,
    }

    #[state_machine(initial = State::idle())]
    impl Hall {
        #[state]
        fn idle() -> Outcome<State> {
            Handled
        }

        #[submachine(machine = door::Door, field = door)]
        fn loading() -> Outcome<State> {
            Handled
        }
    }
}

#[test]
fn depth_crosses_mount_boundary() {
    let mut opened = hall::State::loading(door::State::opened());
    let mut closed = hall::State::loading(door::State::closed());
    let mut idle = hall::State::idle();

    assert_eq!(StateExt::<hall::Hall>::depth(&mut opened), 3);
    assert_eq!(StateExt::<hall::Hall>::depth(&mut idle), 1);
    assert!(opened.same_state(&hall::State::loading(door::State::opened())));
    assert!(!opened.same_state(&closed));
    assert!(!opened.same_state(&idle));
    assert_eq!(opened.transition_path(&mut closed), (1, 1));
    assert_eq!(opened.transition_path(&mut idle), (3, 1));
    assert_eq!(idle.transition_path(&mut opened), (1, 3));
}

mod lamp {
    use sotu::prelude::*;

    use super::trace;

    pub struct Lamp {
        pub lit: bool,
    }

    impl Default for Lamp {
        fn default() -> Self {
            trace::record("lamp created");

            Self { lit: false }
        }
    }

    impl Drop for Lamp {
        fn drop(&mut self) {
            trace::record("lamp dropped");
        }
    }

    #[derive(Event)]
    pub enum Event {}

    #[state_machine(initial = State::shining())]
    impl Lamp {
        #[state(entry_action = turn_on, exit_action = turn_off)]
        fn shining(event: &Event) -> Outcome<State> {
            match *event {}
        }

        #[action]
        fn turn_on(&mut self) {
            self.lit = true;
            trace::record("lamp on");
        }

        #[action]
        fn turn_off(&mut self) {
            self.lit = false;
            trace::record("lamp off");
        }
    }
}

mod house {
    use sotu::prelude::*;

    use super::{door, lamp, trace};

    #[derive(Default)]
    pub struct House {
        pub door: Option<door::Door>,
        pub lamp: Option<lamp::Lamp>,
    }

    pub struct Enter;

    pub struct Leave;

    pub struct Shut;

    pub struct Visit;

    pub struct Peek;

    pub struct Light;

    pub struct Toggle;

    #[derive(Event)]
    pub enum Event {
        Enter(Enter),
        Leave(Leave),
        Shut(Shut),
        Visit(Visit),
        Peek(Peek),
        Light(Light),
        Toggle(Toggle),
    }

    #[state_machine(initial = State::idle())]
    impl House {
        #[state(entry_action = enter_idle, exit_action = exit_idle)]
        fn idle(event: &Event) -> Outcome<State> {
            match event {
                Event::Enter(_) => Transition(Superstate::loading()),
                Event::Light(_) => Transition(Superstate::lighting()),
                _ => Handled,
            }
        }

        #[state(superstate = loading, entry_action = enter_annex, exit_action = exit_annex)]
        fn annex() -> Outcome<State> {
            Super
        }

        #[submachine(machine = door::Door, field = door)]
        fn loading(submachine: &mut door::Door, event: &Event) -> Outcome<State> {
            match event {
                Event::Leave(_) => Transition(State::idle()),
                Event::Shut(_) => Transition(State::loading(door::State::closed())),
                Event::Visit(_) => Transition(State::annex()),
                Event::Peek(_) => {
                    trace::record(match submachine.openings {
                        1 => "door opened once",
                        _ => "door opened otherwise",
                    });

                    submachine.openings += 10;

                    Handled
                }
                _ => Handled,
            }
        }

        #[submachine(machine = lamp::Lamp, field = lamp)]
        fn lighting(event: &Event) -> Outcome<State> {
            match event {
                Event::Toggle(_) => {
                    trace::record("parent saw toggle");

                    Handled
                }
                Event::Leave(_) => Transition(State::idle()),
                _ => Handled,
            }
        }

        #[action]
        fn enter_idle() {
            trace::record("enter idle");
        }

        #[action]
        fn exit_idle() {
            trace::record("exit idle");
        }

        #[action]
        fn enter_annex() {
            trace::record("enter annex");
        }

        #[action]
        fn exit_annex() {
            trace::record("exit annex");
        }
    }
}

mod inner {
    use sotu::prelude::*;

    use super::trace;

    pub struct Inner;

    impl Default for Inner {
        fn default() -> Self {
            trace::record("inner created");

            Self
        }
    }

    #[state_machine(initial = State::leaf())]
    impl Inner {
        #[state(entry_action = enter_leaf)]
        fn leaf() -> Outcome<State> {
            Handled
        }

        #[action]
        fn enter_leaf() {
            trace::record("enter inner leaf");
        }
    }
}

mod outer {
    use sotu::prelude::*;

    use super::{inner, trace};

    pub struct Outer {
        pub inner: Option<inner::Inner>,
    }

    impl Default for Outer {
        fn default() -> Self {
            trace::record("outer created");

            Self { inner: None }
        }
    }

    #[state_machine(initial = Superstate::nested())]
    impl Outer {
        #[submachine(machine = inner::Inner, field = inner)]
        fn nested() -> Outcome<State> {
            Handled
        }
    }
}

mod nest {
    use sotu::prelude::*;

    use super::outer;

    #[derive(Default)]
    pub struct Nest {
        pub outer: Option<outer::Outer>,
    }

    #[state_machine(initial = Superstate::holding())]
    impl Nest {
        #[submachine(machine = outer::Outer, field = outer)]
        fn holding() -> Outcome<State> {
            Handled
        }
    }
}

use house::{Enter, House, Leave, Light, Peek, Shut, Toggle, Visit};

/// Return an initialized `House` in `idle`, with an empty trace.
fn idle_house() -> InitializedStateMachine<House> {
    let machine = House::default().uninitialized_state_machine().init(&mut ());

    trace::take();

    machine
}

#[test]
fn entering_mount_creates_instance_before_child_entries() {
    let mut machine = idle_house();

    machine.handle(Enter, &mut ());

    assert_eq!(
        trace::take(),
        ["exit idle", "door created", "enter frame", "enter opened"]
    );
}

#[test]
fn leaving_mount_drops_instance_after_child_exits() {
    let mut machine = idle_house();

    machine.handle(Enter, &mut ());
    trace::take();
    machine.handle(Leave, &mut ());

    assert_eq!(
        trace::take(),
        ["exit opened", "exit frame", "door dropped", "enter idle"]
    );
}

#[test]
fn instance_field_is_some_exactly_while_mount_is_active() {
    let mut machine = idle_house();

    assert!(machine.door.is_none());

    machine.handle(Enter, &mut ());

    assert!(machine.door.is_some());

    machine.handle(Leave, &mut ());

    assert!(machine.door.is_none());
}

#[test]
fn local_transition_inside_mount_keeps_instance() {
    let mut machine = idle_house();

    machine.handle(Enter, &mut ());
    trace::take();
    machine.handle(Shut, &mut ());

    assert_eq!(trace::take(), ["exit opened", "enter closed"]);
    assert!(matches!(
        machine.state(),
        house::State::Loading(door::State::Closed {})
    ));
    assert!(machine.door.is_some());
}

#[test]
fn parent_leaf_under_mount_keeps_instance() {
    let mut machine = idle_house();

    machine.handle(Enter, &mut ());
    trace::take();
    machine.handle(Visit, &mut ());

    assert_eq!(trace::take(), ["exit opened", "exit frame", "enter annex"]);
    assert!(matches!(machine.state(), house::State::Annex {}));
    assert!(machine.door.is_some());
}

#[test]
fn nested_mount_lives_in_child_field() {
    let machine = nest::Nest::default()
        .uninitialized_state_machine()
        .init(&mut ());

    assert_eq!(
        trace::take(),
        ["outer created", "inner created", "enter inner leaf"]
    );
    assert!(
        machine
            .outer
            .as_ref()
            .is_some_and(|outer| outer.inner.is_some())
    );
}

#[test]
fn empty_event_child_is_never_addressed() {
    let mut machine = idle_house();

    machine.handle(Light, &mut ());

    assert_eq!(trace::take(), ["exit idle", "lamp created", "lamp on"]);
    assert!(machine.lamp.as_ref().is_some_and(|lamp| lamp.lit));

    machine.handle(Toggle, &mut ());
    machine.handle(Toggle, &mut ());

    assert_eq!(trace::take(), ["parent saw toggle", "parent saw toggle"]);

    machine.handle(Leave, &mut ());

    assert_eq!(trace::take(), ["lamp off", "lamp dropped", "enter idle"]);
}

#[test]
fn mount_handler_receives_parent_events() {
    let mut machine = idle_house();

    machine.handle(Enter, &mut ());
    trace::take();
    machine.handle(Peek, &mut ());

    assert_eq!(trace::take(), ["door opened once"]);
    assert!(matches!(
        machine.state(),
        house::State::Loading(door::State::Opened {})
    ));
    assert_eq!(machine.door.as_ref().map(|door| door.openings), Some(11));
}
