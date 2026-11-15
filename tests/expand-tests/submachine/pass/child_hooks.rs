use sotu::prelude::*;

mod lamp {
    use sotu::prelude::*;

    #[derive(Default)]
    pub struct Lamp {
        pub dispatches: u32,
    }

    #[state_machine(
        initial = State::dim(),
        before_dispatch = Self::count,
        after_dispatch = Self::count
    )]
    impl Lamp {
        fn count(&mut self, _node: StateOrSuperstate<'_, State, Superstate>, _event: &()) {
            self.dispatches += 1;
        }

        #[state(superstate = glowing)]
        fn dim() -> Outcome<State> {
            Super
        }

        #[superstate]
        fn glowing() -> Outcome<State> {
            Handled
        }
    }
}

#[derive(Default)]
pub struct Desk {
    lamp: Option<lamp::Lamp>,
}

#[state_machine(initial = Superstate::lighting())]
impl Desk {
    #[submachine(machine = lamp::Lamp, field = lamp)]
    fn lighting() -> Outcome<State> {
        Handled
    }
}

fn main() {
    let mut machine = Desk::default().state_machine();

    machine.step(&mut ());
}
