use sotu::prelude::*;

mod door {
    use sotu::prelude::*;

    #[derive(Default)]
    pub struct Door;

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
    fn idle() -> Outcome<State> {
        Transition(Superstate::loading())
    }

    #[submachine(machine = door::Door, field = door, superstate = guarded)]
    fn loading() -> Outcome<State> {
        Super
    }

    #[superstate(superstate = powered)]
    fn guarded() -> Outcome<State> {
        Super
    }

    #[superstate]
    fn powered(attempts: &mut u32) -> Outcome<State> {
        *attempts += 1;

        Handled
    }
}

fn main() {}
