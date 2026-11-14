use sotu::prelude::*;

mod door {
    use sotu::prelude::*;

    #[derive(Default)]
    pub struct Door;

    #[state_machine(initial = State::opened())]
    impl Door {
        #[state]
        fn opened() -> Outcome<State> {
            Handled
        }
    }
}

#[derive(Default)]
pub struct House {
    door: Option<door::Door>,
}

#[state_machine(initial = State::annex())]
impl House {
    /// A leaf of the parent that keeps the door mounted.
    #[state(superstate = loading)]
    fn annex() -> Outcome<State> {
        Super
    }

    #[submachine(machine = door::Door, field = door)]
    fn loading() -> Outcome<State> {
        Transition(Superstate::loading())
    }
}

fn main() {
    let mut machine = House::default().state_machine();

    machine.step(&mut ());
}
