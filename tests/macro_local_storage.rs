use sotu::prelude::*;

mod lamp {
    use sotu::prelude::*;

    #[derive(Default)]
    pub struct Lamp {
        pub log: Vec<&'static str>,
        pub exited_counter: Option<u32>,
    }

    pub struct Tick;

    pub struct Toggle;

    pub struct Reset;

    #[derive(Event)]
    pub enum Event {
        Tick(Tick),
        Toggle(Toggle),
        Reset(Reset),
    }

    #[state_machine(initial = State::on(0, false))]
    impl Lamp {
        #[state(local_storage(started: bool))]
        fn on(counter: &mut u32, #[default] lit: &mut bool, event: &Event) -> Outcome<State> {
            match event {
                Event::Tick(_) => {
                    *counter += u32::from(*lit);

                    Handled
                }
                Event::Toggle(_) => Transition(State::off()),
                Event::Reset(_) => Transition(State::on(0, false)),
            }
        }

        #[state]
        fn off(event: &Event) -> Outcome<State> {
            match event {
                Event::Tick(_) | Event::Reset(_) => Handled,
                Event::Toggle(_) => Transition(State::on(0, false)),
            }
        }
    }
}

use lamp::{Lamp, Tick};

#[test]
fn constructor_omits_defaulted_fields() {
    let state = lamp::State::on(5, true);

    assert!(matches!(
        state,
        lamp::State::On {
            counter: 5,
            lit: false,
            started: true,
        }
    ));
}

#[test]
fn handlers_receive_local_storage() {
    let mut machine = Lamp::default().state_machine();

    machine.handle(Tick, &mut ());
    machine.handle(Tick, &mut ());
    machine.handle(Tick, &mut ());

    assert!(matches!(
        machine.state(),
        lamp::State::On {
            counter: 0,
            lit: false,
            ..
        }
    ));
}

#[test]
fn local_storage_is_written_by_the_handler() {
    let mut machine = Lamp::default().state_machine();

    let lamp::State::On { lit, .. } = machine.state_mut() else {
        panic!("the lamp should be on");
    };

    *lit = true;

    machine.handle(Tick, &mut ());

    let lamp::State::On { counter, .. } = machine.state() else {
        panic!("the lamp should be on");
    };

    assert_eq!(*counter, 1);
    assert!(machine.log.is_empty());
    assert!(machine.exited_counter.is_none());
}
