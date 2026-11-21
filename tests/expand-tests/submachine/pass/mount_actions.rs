use sotu::prelude::*;

mod door {
    use sotu::prelude::*;

    #[derive(Default)]
    pub struct Door {
        pub openings: u32,
    }

    /// The door lists no events, so it can be mounted under any parent.
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
    visits: u32,
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

    /// The mount's actions run around the door's creation and release.
    #[submachine(machine = door::Door, field = door, entry_action = greet, exit_action = part)]
    fn loading(event: &Event) -> Outcome<State> {
        match event {
            Event::Enter => Handled,
            Event::Leave => Transition(State::idle()),
        }
    }

    #[action]
    fn greet(submachine: &mut door::Door) {
        submachine.openings += 1;
    }

    #[action]
    fn part(&mut self) {
        self.visits += 1;
    }
}

fn main() {
    let mut machine = House::default().state_machine();

    machine.handle(Event::Enter, &mut ());
    machine.handle(Event::Leave, &mut ());
}
