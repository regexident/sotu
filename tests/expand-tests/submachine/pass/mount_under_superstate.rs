use sotu::prelude::*;

mod fan {
    use sotu::prelude::*;

    #[derive(Default)]
    pub struct Fan;

    #[state_machine(initial = State::slow())]
    impl Fan {
        #[state(superstate = spinning)]
        fn slow() -> Outcome<State> {
            Super
        }

        #[state(superstate = spinning)]
        fn fast() -> Outcome<State> {
            Super
        }

        #[superstate]
        fn spinning() -> Outcome<State> {
            Handled
        }
    }
}

#[derive(Default)]
pub struct Room {
    fan: fan::Fan,
}

#[state_machine(initial = State::cooling(fan::State::slow()))]
impl Room {
    #[state(superstate = powered)]
    fn lit() -> Outcome<State> {
        Super
    }

    /// The fan keeps its storage between activations.
    #[submachine(machine = fan::Fan, field = fan, name = Cooling, superstate = powered)]
    fn cool() -> Outcome<State> {
        Transition(State::cooling(fan::State::fast()))
    }

    #[superstate]
    fn powered() -> Outcome<State> {
        Transition(State::lit())
    }
}

fn main() {
    let mut machine = Room::default().state_machine();

    machine.step(&mut ());
}
