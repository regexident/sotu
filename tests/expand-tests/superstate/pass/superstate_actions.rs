use sotu::prelude::*;

mod borrowing {
    use sotu::prelude::*;

    #[derive(Default)]
    pub struct Heater {
        pub seen: u32,
    }

    #[state_machine(initial = State::heating(0, 0))]
    impl Heater {
        #[state(superstate = powered, local_storage(level: u32, spare: u8))]
        fn heating() -> Outcome<State> {
            Super
        }

        #[state]
        fn cold() -> Outcome<State> {
            Handled
        }

        #[superstate(entry_action = power_on, exit_action = power_off)]
        fn powered(level: &mut u32) -> Outcome<State> {
            *level += 1;

            Transition(State::cold())
        }

        #[action]
        fn power_on(&mut self, level: &mut u32) {
            *level += 1;
            self.seen = *level;
        }

        #[action]
        fn power_off(&mut self, level: &u32) {
            self.seen = *level;
        }
    }
}

mod field_less {
    use sotu::prelude::*;

    #[derive(Default)]
    pub struct Fan {
        pub starts: u32,
    }

    #[state_machine(initial = State::spinning())]
    impl Fan {
        #[state(superstate = running)]
        fn spinning() -> Outcome<State> {
            Super
        }

        #[superstate(superstate = powered, exit_action = stop)]
        fn running() -> Outcome<State> {
            Super
        }

        #[superstate(entry_action = start)]
        fn powered() -> Outcome<State> {
            Handled
        }

        #[action]
        fn start(&mut self) {
            self.starts += 1;
        }

        #[action]
        fn stop() {}
    }
}

fn main() {
    let mut heater = borrowing::Heater::default().state_machine();
    let mut fan = field_less::Fan::default().state_machine();

    heater.step(&mut ());
    fan.step(&mut ());
}
