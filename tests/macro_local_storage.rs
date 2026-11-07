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
        #[state(
            entry_action = enter_on,
            exit_action = leave_on,
            local_storage(started: bool)
        )]
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

        #[state(entry_action = enter_off)]
        fn off(event: &Event) -> Outcome<State> {
            match event {
                Event::Tick(_) | Event::Reset(_) => Handled,
                Event::Toggle(_) => Transition(State::on(0, false)),
            }
        }

        #[action]
        fn enter_on(&mut self, lit: &mut bool) {
            *lit = true;
            self.log.push("enter on");
        }

        #[action]
        fn leave_on(&mut self, counter: &mut u32) {
            *counter += 100;
            self.exited_counter = Some(*counter);
            self.log.push("leave on");
        }

        #[action]
        fn enter_off(&mut self) {
            self.log.push("enter off");
        }
    }
}

mod door {
    use sotu::prelude::*;

    #[derive(Default)]
    pub struct Door {
        pub ticks: u32,
    }

    #[state_machine(initial = State::idle())]
    impl Door {
        #[state(name = Idle)]
        fn waiting(&mut self) -> Outcome<State> {
            self.ticks += 1;

            Handled
        }
    }
}

use door::Door;
use lamp::{Lamp, Reset, Tick, Toggle};

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
        lamp::State::On { counter: 3, .. }
    ));
}

#[test]
fn entry_and_exit_actions_run_in_order() {
    let mut machine = Lamp::default().state_machine();

    machine.handle(Toggle, &mut ());

    assert_eq!(machine.log, ["enter on", "leave on", "enter off"]);
}

#[test]
fn exit_action_changes_do_not_reach_target() {
    let mut machine = Lamp::default().state_machine();

    machine.handle(Tick, &mut ());
    machine.handle(Reset, &mut ());

    assert_eq!(machine.exited_counter, Some(101));
    assert!(matches!(
        machine.state(),
        lamp::State::On {
            counter: 0,
            lit: true,
            ..
        }
    ));
}

#[test]
fn renamed_state_uses_new_name() {
    let mut machine = Door::default().state_machine();

    assert!(matches!(door::State::idle(), door::State::Idle {}));

    machine.step(&mut ());

    assert_eq!(machine.ticks, 1);
    assert!(matches!(machine.state(), door::State::Idle {}));
}
