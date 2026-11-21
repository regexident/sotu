use sotu::prelude::*;

mod door {
    use sotu::prelude::*;

    pub enum DoorExit {
        Closed,
    }

    #[derive(Default)]
    pub struct Door {
        pub openings: u32,
    }

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
    visits: u32,
    door: Option<door::Door>,
}

#[state_machine(initial = State::idle())]
impl House {
    #[state]
    fn idle() -> Outcome<State> {
        Transition(Superstate::loading())
    }

    #[submachine(machine = door::Door, field = door, exit_handler = door_exited)]
    fn loading() -> Outcome<State> {
        Handled
    }

    #[exit_handler]
    fn door_exited(
        &mut self,
        submachine: &mut door::Door,
        exit: door::DoorExit,
    ) -> Outcome<State> {
        self.visits += submachine.openings;

        match exit {
            door::DoorExit::Closed => Transition(State::idle()),
        }
    }
}

fn main() {}
