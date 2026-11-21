use sotu::prelude::*;

mod lamp {
    use sotu::prelude::*;

    #[derive(Default)]
    pub struct Lamp {
        pub toggles: u32,
    }

    #[state_machine(initial = State::off())]
    impl Lamp {
        #[state(entry_action = reset)]
        fn off(&mut self) -> Outcome<State> {
            self.toggles += 1;

            Handled
        }

        #[action]
        fn reset(&mut self) {
            self.toggles = 0;
        }
    }
}

#[derive(Default)]
pub struct Room {
    lamp: Option<lamp::Lamp>,
}

#[state_machine(initial = Superstate::lit(), context = u32)]
impl Room {
    #[submachine(machine = lamp::Lamp, field = lamp, context = sotu::event::unit_context)]
    fn lit(context: &mut u32) -> Outcome<State> {
        *context += 1;

        Handled
    }
}

fn main() {
    let mut context = 0;
    let mut machine = Room::default().state_machine();

    machine.step(&mut context);
}
