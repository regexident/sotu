use sotu::prelude::*;

mod renamed {
    use sotu::prelude::*;

    pub struct Start;

    #[derive(Event)]
    pub enum Event {
        Start(Start),
    }

    pub struct Runner;

    #[state_machine(
        initial = Phase::idle(),
        state(name = Phase),
        superstate(name = Group)
    )]
    impl Runner {
        #[state(superstate = active)]
        fn idle(event: &Event) -> Outcome<Phase> {
            match event {
                Event::Start(_) => Super,
            }
        }

        #[state(superstate = active)]
        fn running() -> Outcome<Phase> {
            Super
        }

        #[superstate]
        fn active(event: &Event) -> Outcome<Phase> {
            match event {
                Event::Start(_) => Transition(Phase::running()),
            }
        }
    }
}

mod custom {
    use sotu::prelude::*;

    pub struct Brighten;

    #[derive(Event)]
    pub enum Event {
        Brighten(Brighten),
    }

    pub enum Light {
        Off {},
        On { level: u8 },
    }

    pub struct Lamp;

    #[state_machine(initial = Light::Off {}, state(custom, name = Light))]
    impl Lamp {
        #[state]
        fn off(event: &Event) -> Outcome<Light> {
            match event {
                Event::Brighten(_) => Transition(Light::On { level: 3 }),
            }
        }

        #[state]
        fn on(level: &mut u8, event: &Event) -> Outcome<Light> {
            match event {
                Event::Brighten(_) => {
                    *level += 1;

                    Handled
                }
            }
        }
    }
}

mod inner {
    use sotu::prelude::*;

    pub(crate) struct Flip;

    #[derive(Event)]
    pub(crate) enum Event {
        Flip(Flip),
    }

    pub(crate) struct Switch;

    #[state_machine(initial = State::off(), visibility = pub(crate))]
    impl Switch {
        #[state(superstate = wired)]
        fn off(event: &Event) -> Outcome<State> {
            match event {
                Event::Flip(_) => Transition(State::on()),
            }
        }

        #[state(superstate = wired)]
        fn on(event: &Event) -> Outcome<State> {
            match event {
                Event::Flip(_) => Transition(State::off()),
            }
        }

        #[superstate(initial = State::on())]
        fn wired() -> Outcome<State> {
            Handled
        }
    }
}

#[test]
fn renamed_state_type_is_used() {
    let mut machine = renamed::Runner.state_machine();

    let _: &renamed::Phase = machine.state();

    assert!(matches!(machine.state(), renamed::Phase::Idle {}));

    machine.handle(renamed::Start, &mut ());

    assert!(matches!(machine.state(), renamed::Phase::Running {}));
}

#[test]
fn custom_state_enum_drives_machine() {
    let mut machine = custom::Lamp.state_machine();

    assert!(matches!(machine.state(), custom::Light::Off {}));

    machine.handle(custom::Brighten, &mut ());

    assert!(matches!(machine.state(), custom::Light::On { level: 3 }));

    machine.handle(custom::Brighten, &mut ());

    assert!(matches!(machine.state(), custom::Light::On { level: 4 }));
}

#[test]
fn restricted_visibility_compiles_in_module() {
    let mut machine = inner::Switch.state_machine();

    assert!(matches!(machine.state(), inner::State::Off {}));

    machine.handle(inner::Flip, &mut ());

    assert!(matches!(machine.state(), inner::State::On {}));
    assert!(matches!(inner::State::off(), inner::State::Off {}));
    assert!(matches!(inner::Superstate::wired(), inner::State::On {}));
}
