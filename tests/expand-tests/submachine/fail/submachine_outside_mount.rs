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
    door: Option<door::Door>,
}

#[state_machine(initial = State::idle())]
impl House {
    #[state]
    fn idle(submachine: &mut door::Door) -> Outcome<State> {
        submachine.openings += 1;

        Transition(Superstate::loading())
    }

    #[submachine(machine = door::Door, field = door)]
    fn loading(submachine: &mut door::Door) -> Outcome<State> {
        submachine.openings += 1;

        Handled
    }
}

fn main() {}
