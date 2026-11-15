use sotu::prelude::*;

mod pump {
    use sotu::prelude::*;

    #[derive(Default)]
    pub struct Pump {
        pub transitions: u32,
    }

    #[state_machine(
        initial = State::off(),
        before_transition = Self::count,
        after_transition = Self::count
    )]
    impl Pump {
        fn count(&mut self, _source: &State, _target: &State) {
            self.transitions += 1;
        }

        #[state]
        fn off() -> Outcome<State> {
            Transition(State::on())
        }

        #[state]
        fn on() -> Outcome<State> {
            Transition(State::off())
        }
    }
}

#[derive(Default)]
pub struct Station {
    pump: Option<pump::Pump>,
}

#[state_machine(initial = Superstate::pumping())]
impl Station {
    #[submachine(machine = pump::Pump, field = pump)]
    fn pumping() -> Outcome<State> {
        Handled
    }

    #[state]
    fn closed() -> Outcome<State> {
        Handled
    }
}

fn main() {
    let mut machine = Station::default().state_machine();

    machine.step(&mut ());
}
