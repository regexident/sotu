use sotu::prelude::*;

mod door {
    use sotu::prelude::*;

    #[derive(Default)]
    pub struct Door {
        pub openings: u32,
    }

    #[state_machine(initial = State::closed())]
    impl Door {
        #[state]
        fn closed() -> Outcome<State> {
            Handled
        }
    }
}

#[derive(Default)]
pub struct House {
    visits: u32,
    door: Option<door::Door>,
}

#[state_machine(initial = State::idle())]
impl House {
    #[state]
    fn idle() -> Outcome<State> {
        Transition(Superstate::loading())
    }

    #[submachine(machine = door::Door, field = door, entry_action = greet)]
    fn loading() -> Outcome<State> {
        Handled
    }

    #[action]
    fn greet(&mut self, submachine: &mut door::Door) {
        self.visits += 1;
        submachine.openings += 1;
    }
}

fn main() {}
