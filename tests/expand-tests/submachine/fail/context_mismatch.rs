use sotu::prelude::*;

mod door {
    use sotu::prelude::*;

    #[derive(Default)]
    pub struct Door;

    #[state_machine(initial = State::closed(), context = u8)]
    impl Door {
        #[state]
        fn closed(context: &mut u8) -> Outcome<State> {
            *context += 1;

            Handled
        }
    }
}

#[derive(Default)]
pub struct House {
    door: Option<door::Door>,
}

#[state_machine(initial = State::idle(), context = u16)]
impl House {
    #[state]
    fn idle(context: &mut u16) -> Outcome<State> {
        *context += 1;

        Transition(Superstate::loading())
    }

    #[submachine(machine = door::Door, field = door)]
    fn loading() -> Outcome<State> {
        Handled
    }
}

fn main() {}
