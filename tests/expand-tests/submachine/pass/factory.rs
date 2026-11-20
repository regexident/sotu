use sotu::prelude::*;

mod door {
    use sotu::prelude::*;

    /// The door has no `Default`, so it is created by the parent's factory.
    pub struct Door {
        pub id: u32,
    }

    #[derive(Event)]
    pub enum Event {}

    #[state_machine(initial = State::opened())]
    impl Door {
        #[state]
        fn opened(event: &Event) -> Outcome<State> {
            match *event {}
        }
    }
}

pub enum Event {
    Enter,
    Leave,
}

#[derive(Default)]
pub struct House {
    next_id: u32,
    door: Option<door::Door>,
}

#[state_machine(initial = State::idle())]
impl House {
    #[state]
    fn idle(event: &Event) -> Outcome<State> {
        match event {
            Event::Enter => Transition(Superstate::loading()),
            Event::Leave => Handled,
        }
    }

    #[submachine(machine = door::Door, field = door, factory = Self::make_door)]
    fn loading(event: &Event) -> Outcome<State> {
        match event {
            Event::Enter => Handled,
            Event::Leave => Transition(State::idle()),
        }
    }

    fn make_door(&mut self, _context: &mut ()) -> door::Door {
        self.next_id += 1;

        door::Door { id: self.next_id }
    }
}

fn main() {
    let mut machine = House::default().state_machine();

    machine.handle(Event::Enter, &mut ());
    machine.handle(Event::Leave, &mut ());
}
