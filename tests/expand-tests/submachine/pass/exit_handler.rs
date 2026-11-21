use sotu::prelude::*;

mod door {
    use sotu::prelude::*;

    #[derive(Default)]
    pub struct Door {
        pub openings: u32,
    }

    pub struct Close;

    #[derive(Event)]
    pub enum Event {
        Close(Close),
    }

    pub enum DoorExit {
        Closed,
    }

    #[state_machine(initial = State::opened(), exit = DoorExit)]
    impl Door {
        #[state(superstate = frame)]
        fn opened(event: &Event) -> Outcome<State, DoorExit> {
            match event {
                Event::Close(_) => Super,
            }
        }

        /// The door exits from its superstate as well as from its leaves.
        #[superstate]
        fn frame(event: &Event) -> Outcome<State, DoorExit> {
            match event {
                Event::Close(_) => Exit(DoorExit::Closed),
            }
        }
    }
}

pub struct Start;

#[derive(Event)]
pub enum Event {
    Start(Start),
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
            Event::Start(_) => Transition(Superstate::loading()),
            Event::Close(_) => Handled,
        }
    }

    /// The door's exits go to `door_exited`.
    #[submachine(machine = door::Door, field = door, exit_handler = door_exited)]
    fn loading(event: &Event) -> Outcome<State> {
        match event {
            Event::Start(_) => Handled,
            Event::Close(_) => Super,
        }
    }

    #[exit_handler]
    fn door_exited(submachine: &mut door::Door, exit: door::DoorExit) -> Outcome<State> {
        match exit {
            door::DoorExit::Closed if submachine.openings > 0 => Transition(State::idle()),
            door::DoorExit::Closed => Super,
        }
    }
}

fn main() {
    let mut machine = House::default().state_machine();

    machine.handle(Start, &mut ());
    machine.handle(door::Close, &mut ());
}
