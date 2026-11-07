use sotu::prelude::*;

pub enum Event {
    Flip,
}

mod entry_only {
    use sotu::prelude::*;

    use super::Event;

    #[derive(Default)]
    pub struct Lamp {
        pub flips: u32,
    }

    #[state_machine(initial = State::on(0, false))]
    impl Lamp {
        #[state(entry_action = light, local_storage(started: bool))]
        fn on(&mut self, counter: &mut u32, #[default] lit: &mut bool, event: &Event) -> Outcome<State> {
            match event {
                Event::Flip if *lit => {
                    *counter += 1;
                    self.flips = *counter;

                    Transition(State::off())
                }
                Event::Flip => Handled,
            }
        }

        #[state]
        fn off(event: &Event) -> Outcome<State> {
            match event {
                Event::Flip => Transition(State::on(0, false)),
            }
        }

        #[action]
        fn light(&mut self, lit: &mut bool) {
            self.flips += 1;
            *lit = true;
        }
    }
}

mod exit_only {
    use sotu::prelude::*;

    use super::Event;

    #[derive(Default)]
    pub struct Latch {
        pub total: u32,
    }

    #[state_machine(initial = State::first(1))]
    impl Latch {
        #[state(exit_action = record)]
        fn first(amount: &mut u32, event: &Event) -> Outcome<State> {
            match event {
                Event::Flip => Transition(State::second(*amount + 1)),
            }
        }

        #[state(exit_action = record)]
        fn second(amount: &mut u32, #[default] extra: &mut u8, event: &Event) -> Outcome<State> {
            *extra += 1;

            match event {
                Event::Flip => Transition(State::first(*amount)),
            }
        }

        #[action]
        fn record(&mut self, amount: &mut u32) {
            self.total += *amount;
        }
    }
}

fn main() {
    let mut lamp = entry_only::Lamp::default().state_machine();
    let mut latch = exit_only::Latch::default().state_machine();

    lamp.handle(Event::Flip, &mut ());
    latch.handle(Event::Flip, &mut ());

    assert_eq!(lamp.flips, 1);
    assert_eq!(latch.total, 1);
}
