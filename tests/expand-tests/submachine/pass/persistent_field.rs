use sotu::prelude::*;

mod log {
    use sotu::prelude::*;

    /// The log has no `Default`, since a persistent field is never created
    /// by the machine.
    pub struct Log {
        pub entries: u32,
    }

    #[derive(Event)]
    pub enum Event {}

    #[state_machine(initial = State::writing())]
    impl Log {
        #[state(entry_action = count)]
        fn writing(event: &Event) -> Outcome<State> {
            match *event {}
        }

        #[action]
        fn count(&mut self) {
            self.entries += 1;
        }
    }
}

pub enum Event {
    Enter,
    Leave,
}

pub struct House {
    log: log::Log,
}

#[state_machine(initial = State::idle())]
impl House {
    #[state]
    fn idle(event: &Event) -> Outcome<State> {
        match event {
            Event::Enter => Transition(Superstate::logging()),
            Event::Leave => Handled,
        }
    }

    #[submachine(machine = log::Log, field = log)]
    fn logging(event: &Event) -> Outcome<State> {
        match event {
            Event::Enter => Handled,
            Event::Leave => Transition(State::idle()),
        }
    }
}

fn main() {
    let mut machine = House {
        log: log::Log { entries: 0 },
    }
    .state_machine();

    machine.handle(Event::Enter, &mut ());
    machine.handle(Event::Leave, &mut ());
}
