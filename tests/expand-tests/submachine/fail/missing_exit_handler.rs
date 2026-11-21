use sotu::prelude::*;

mod door {
    use sotu::prelude::*;

    pub enum DoorExit {
        Closed,
    }

    #[derive(Default)]
    pub struct Door;

    #[state_machine(initial = State::closed(), exit = DoorExit)]
    impl Door {
        #[state]
        fn closed() -> Outcome<State, DoorExit> {
            Exit(DoorExit::Closed)
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

    #[submachine(machine = door::Door, field = door, exit_handler = door_left)]
    fn loading() -> Outcome<State> {
        Handled
    }
}

fn main() {}
