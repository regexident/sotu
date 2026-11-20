use sotu::prelude::*;

mod door {
    use sotu::prelude::*;

    pub struct Door;

    #[state_machine(initial = State::closed())]
    impl Door {
        #[state]
        fn closed() -> Outcome<State> {
            Handled
        }
    }
}

pub struct House {
    door: door::Door,
}

#[state_machine(initial = State::idle())]
impl House {
    #[state]
    fn idle() -> Outcome<State> {
        Transition(Superstate::loading())
    }

    #[submachine(machine = door::Door, field = door, factory = Self::make_door)]
    fn loading() -> Outcome<State> {
        Handled
    }

    fn make_door(&mut self, _context: &mut ()) -> door::Door {
        door::Door
    }
}

fn main() {}
