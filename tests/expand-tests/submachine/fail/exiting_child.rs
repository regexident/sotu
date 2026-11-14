use sotu::prelude::*;

mod door {
    use sotu::prelude::*;

    pub enum Lock {
        Engaged,
    }

    #[derive(Default)]
    pub struct Door;

    #[state_machine(initial = State::closed(), exit = Lock)]
    impl Door {
        #[state]
        fn closed() -> Outcome<State, Lock> {
            Exit(Lock::Engaged)
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

    #[submachine(machine = door::Door, field = door)]
    fn loading() -> Outcome<State> {
        Handled
    }
}

fn main() {}
