use sotu::prelude::*;

mod door {
    use sotu::prelude::*;

    #[derive(Default)]
    pub struct Door;

    pub struct Open;

    pub struct Close;

    #[derive(Event)]
    pub enum Event {
        Open(Open),
        Close(Close),
    }

    #[state_machine(initial = State::opened())]
    impl Door {
        #[state(superstate = frame)]
        fn opened(event: &Event) -> Outcome<State> {
            match event {
                Event::Close(_) => Transition(State::closed()),
                Event::Open(_) => Super,
            }
        }

        #[state(superstate = frame)]
        fn closed(event: &Event) -> Outcome<State> {
            match event {
                Event::Open(_) => Transition(State::opened()),
                Event::Close(_) => Super,
            }
        }

        #[superstate]
        fn frame() -> Outcome<State> {
            Super
        }
    }
}

pub struct Enter;

#[derive(Event)]
pub enum Event {
    Enter(Enter),
    Open(door::Open),
    Close(door::Close),
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
            Event::Enter(_) => Transition(Superstate::loading()),
            _ => Handled,
        }
    }

    /// Events the door lists reach the door before the mount handler.
    #[submachine(machine = door::Door, field = door)]
    fn loading(event: &Event) -> Outcome<State> {
        match event {
            Event::Enter(_) => Transition(State::idle()),
            _ => Handled,
        }
    }
}

fn main() {
    let mut machine = House::default().state_machine();

    machine.handle(Enter, &mut ());
    machine.handle(door::Close, &mut ());
    machine.handle(door::Open, &mut ());
}
