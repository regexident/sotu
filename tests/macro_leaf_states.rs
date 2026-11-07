use sotu::prelude::*;

mod switch {
    use sotu::prelude::*;

    #[derive(Default)]
    pub struct Switch {
        pub flips: u32,
    }

    pub struct Flip;

    pub struct Noop;

    #[derive(Event)]
    pub enum Event {
        Flip(Flip),
        Noop(Noop),
    }

    #[state_machine(initial = State::off())]
    impl Switch {
        #[state]
        fn off(&mut self, event: &Event) -> Outcome<State> {
            match event {
                Event::Flip(_) => {
                    self.flips += 1;

                    Transition(State::on())
                }
                Event::Noop(_) => Super,
            }
        }

        #[state]
        fn on(&mut self, event: &Event) -> Outcome<State> {
            match event {
                Event::Flip(_) => {
                    self.flips += 1;

                    Transition(State::off())
                }
                Event::Noop(_) => Super,
            }
        }
    }
}

mod latch {
    use sotu::prelude::*;

    use super::switch::Event;

    #[derive(Default)]
    pub struct Latch;

    #[state_machine(initial = State::off())]
    impl Latch {
        #[state]
        fn off(event: &Event) -> Outcome<State> {
            match event {
                Event::Flip(_) => Transition(State::on()),
                Event::Noop(_) => Super,
            }
        }

        #[state]
        fn on(event: &Event) -> Outcome<State> {
            match event {
                Event::Flip(_) => Transition(State::off()),
                Event::Noop(_) => Super,
            }
        }
    }
}

mod ticker {
    use sotu::prelude::*;

    #[derive(Default)]
    pub struct Ticker {
        pub ticks: u32,
    }

    #[state_machine(initial = State::ticking())]
    impl Ticker {
        #[state]
        fn ticking(&mut self) -> Outcome<State> {
            self.ticks += 1;

            Handled
        }
    }
}

use latch::Latch;
use switch::{Event, Flip, Noop, Switch};
use ticker::Ticker;

#[test]
fn switch_flips_between_states() {
    let mut machine = Switch::default().state_machine();

    machine.handle(Flip, &mut ());

    assert!(matches!(machine.state(), switch::State::On {}));

    machine.handle(Flip, &mut ());

    assert_eq!(machine.flips, 2);
    assert!(matches!(machine.state(), switch::State::Off {}));
}

#[test]
fn unhandled_event_is_dropped() {
    let mut machine = Switch::default().state_machine();

    assert!(machine.handle(Noop, &mut ()).is_none());

    assert_eq!(machine.flips, 0);
    assert!(matches!(machine.state(), switch::State::Off {}));
}

#[test]
fn event_enum_and_payload_are_equivalent() {
    let mut from_enum = Switch::default().state_machine();
    let mut from_payload = Switch::default().state_machine();

    from_enum.handle(Event::Flip(Flip), &mut ());
    from_payload.handle(Flip, &mut ());

    assert_eq!(from_enum.flips, 1);
    assert_eq!(from_payload.flips, 1);
    assert!(matches!(from_enum.state(), switch::State::On {}));
    assert!(matches!(from_payload.state(), switch::State::On {}));
}

#[test]
fn handler_without_receiver_is_called() {
    let mut machine = Latch.state_machine();

    machine.handle(Flip, &mut ());

    assert!(matches!(machine.state(), latch::State::On {}));

    machine.handle(Flip, &mut ());

    assert!(matches!(machine.state(), latch::State::Off {}));
}

#[test]
fn unit_event_machine_steps() {
    let mut machine = Ticker::default().state_machine();

    machine.step(&mut ());
    machine.step(&mut ());
    machine.step(&mut ());

    assert_eq!(machine.ticks, 3);
}
