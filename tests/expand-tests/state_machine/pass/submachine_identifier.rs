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
    Knock,
}

#[derive(Default)]
pub struct House {
    door: Option<door::Door>,
}

#[state_machine(initial = State::idle(), submachine_identifier = child)]
impl House {
    #[state]
    fn idle(event: &Event) -> Outcome<State> {
        match event {
            Event::Enter => Transition(Superstate::loading()),
            Event::Knock => Handled,
        }
    }

    #[submachine(machine = door::Door, field = door)]
    fn loading(child: &mut door::Door, event: &Event) -> Outcome<State> {
        match event {
            Event::Enter => Handled,
            Event::Knock => {
                child.openings += 1;

                Handled
            }
        }
    }
}

fn main() {
    let mut machine = House::default().state_machine();

    machine.handle(Event::Enter, &mut ());
    machine.handle(Event::Knock, &mut ());
}
