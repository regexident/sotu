use sotu::prelude::*;

mod door {
    use sotu::prelude::*;

    #[derive(Default)]
    pub struct Door;

    #[state_machine(initial = State::closed(), awaitable)]
    impl Door {
        #[state]
        fn closed() -> Outcome<State> {
            Handled
        }
    }
}

#[derive(Default)]
pub struct Hall {
    door: Option<door::Door>,
}

#[state_machine(initial = State::idle())]
impl Hall {
    #[state]
    fn idle() -> Outcome<State> {
        Transition(Superstate::loading())
    }

    #[submachine(machine = door::Door, field = door)]
    fn loading() -> Outcome<State> {
        Handled
    }
}

fn main() {}
