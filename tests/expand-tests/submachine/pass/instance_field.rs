use sotu::prelude::*;

mod door {
    use sotu::prelude::*;

    #[derive(Default)]
    pub struct Door {
        pub openings: u32,
    }

    #[state_machine(initial = State::opened())]
    impl Door {
        #[state(entry_action = count)]
        fn opened() -> Outcome<State> {
            Handled
        }

        #[action]
        fn count(&mut self) {
            self.openings += 1;
        }
    }
}

pub enum Event {
    Enter,
    Peek,
}

#[derive(Default)]
pub struct House {
    door: Option<door::Door>,
}

#[state_machine(initial = State::idle())]
impl House {
    #[state]
    fn idle(event: &Event) -> Outcome<State> {
        match event {
            Event::Enter => Transition(Superstate::loading()),
            Event::Peek => Handled,
        }
    }

    /// The door is mounted while it loads.
    #[submachine(machine = door::Door, field = door)]
    fn loading(submachine: &mut door::Door, event: &Event) -> Outcome<State> {
        match event {
            Event::Enter => Handled,
            Event::Peek if submachine.openings > 1 => Transition(State::idle()),
            Event::Peek => Handled,
        }
    }
}

fn main() {
    let mut machine = House::default().state_machine();

    machine.handle(Event::Enter, &mut ());
    machine.handle(Event::Peek, &mut ());
}
