use sotu::prelude::*;

pub enum Event {
    Press,
    Finish,
}

pub enum Done {
    Finished,
}

#[derive(Default)]
pub struct Button {
    pub presses: u32,
}

#[state_machine(initial = State::idle(), exit = Done, awaitable)]
impl Button {
    #[state(entry_action = count)]
    fn idle(event: &Event) -> Outcome<State, Done> {
        match event {
            Event::Press => Transition(State::idle()),
            Event::Finish => Exit(Done::Finished),
        }
    }

    #[action]
    fn count(&mut self) {
        self.presses += 1;
    }
}

fn main() {
    let _machine = Button::default().state_machine();
}
