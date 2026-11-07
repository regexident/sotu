use sotu::prelude::*;

#[derive(Default)]
pub struct Probe {
    pub seen: u32,
}

#[state_machine(initial = State::measuring(1, 2))]
impl Probe {
    #[state(entry_action = prepare, exit_action = sample)]
    fn measuring(&mut self, first: &mut u32, second: &mut u32) -> Outcome<State> {
        self.seen += *first + *second;

        Transition(State::idle())
    }

    #[state(entry_action = prepare_nothing)]
    fn idle() -> Outcome<State> {
        Handled
    }

    #[action]
    fn prepare(second: &mut u32, first: &u32) {
        *second += *first;
    }

    #[action]
    fn prepare_nothing() {}

    #[action]
    fn sample(&mut self, second: &u32) {
        self.seen += *second;
    }
}

mod no_receiver {
    use sotu::prelude::*;

    #[derive(Default)]
    pub struct Silent;

    #[state_machine(initial = State::only(0))]
    impl Silent {
        #[state(entry_action = bump, exit_action = bump)]
        fn only(count: &mut u32) -> Outcome<State> {
            *count += 1;

            Handled
        }

        #[action]
        fn bump(count: &mut u32) {
            *count += 1;
        }
    }
}

fn main() {
    let mut probe = Probe::default().state_machine();
    let mut silent = no_receiver::Silent.state_machine();

    probe.step(&mut ());
    silent.step(&mut ());

    assert_eq!(probe.seen, 7);
}
