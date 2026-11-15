use sotu::prelude::*;

mod bell {
    use sotu::prelude::*;

    #[derive(Default)]
    pub struct Bell;

    pub struct Ring;

    #[derive(Event)]
    pub enum Event {
        Ring(Ring),
    }

    #[state_machine(initial = State::silent())]
    impl Bell {
        #[state]
        fn silent(event: &Event) -> Outcome<State> {
            match event {
                Event::Ring(_) => Handled,
            }
        }
    }
}

pub struct Enter;

#[derive(Event)]
pub enum Event {
    Enter(Enter),
}

#[derive(Default)]
pub struct House {
    bell: Option<bell::Bell>,
}

#[state_machine(initial = State::idle())]
impl House {
    #[state]
    fn idle(event: &Event) -> Outcome<State> {
        match event {
            Event::Enter(_) => Transition(Superstate::ringing()),
        }
    }

    #[submachine(machine = bell::Bell, field = bell)]
    fn ringing() -> Outcome<State> {
        Handled
    }
}

fn main() {}
