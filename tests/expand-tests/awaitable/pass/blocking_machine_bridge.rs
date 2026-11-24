use sotu::prelude::*;

mod door {
    use sotu::prelude::*;

    #[derive(Default)]
    pub struct Door;

    #[derive(Event)]
    pub enum Event {}

    #[state_machine(initial = State::opened())]
    impl Door {
        #[state(superstate = frame)]
        fn opened(event: &Event) -> Outcome<State> {
            match *event {}
        }

        #[superstate]
        fn frame(event: &Event) -> Outcome<State> {
            match *event {}
        }
    }
}

#[derive(Default)]
pub struct Hall {
    pub door: Option<door::Door>,
    pub visits: u32,
}

#[state_machine(initial = State::idle(0))]
impl Hall {
    #[state(superstate = lobby)]
    fn idle(visits: &mut u32) -> Outcome<State> {
        *visits += 1;

        Super
    }

    #[submachine(machine = door::Door, field = door, superstate = region)]
    fn loading() -> Outcome<State> {
        Handled
    }

    #[superstate(superstate = region)]
    fn lobby(visits: &u32) -> Outcome<State> {
        if *visits > 1 { Handled } else { Super }
    }

    #[superstate]
    fn region() -> Outcome<State> {
        Handled
    }
}

fn main() {
    let _machine = Hall::default().state_machine();
}
