use sotu::prelude::*;

mod door {
    use sotu::prelude::*;

    pub struct Push;

    pub struct Close;

    #[derive(Event)]
    pub enum Event {
        Push(Push),
        Close(Close),
    }

    pub enum Lock {
        Engaged,
    }

    #[derive(Default)]
    pub struct Door {
        pub pushes: u32,
    }

    #[state_machine(initial = State::opened(), exit = Lock)]
    impl Door {
        #[state(superstate = frame)]
        fn opened(&mut self, event: &Event) -> Outcome<State, Lock> {
            match event {
                Event::Push(_) => {
                    self.pushes += 1;

                    Super
                }
                Event::Close(_) => Exit(Lock::Engaged),
            }
        }

        #[superstate]
        fn frame() -> Outcome<State, Lock> {
            Handled
        }
    }
}

pub struct Start;

#[derive(Event)]
pub enum Event {
    Start(Start),
    Push(door::Push),
    Close(door::Close),
}

#[derive(Default)]
pub struct Hall {
    pub door: Option<door::Door>,
    pub entries: u32,
}

#[state_machine(initial = State::idle(), context = u32, awaitable)]
impl Hall {
    #[state]
    fn idle(event: &Event) -> Outcome<State> {
        match event {
            Event::Start(_) => Transition(Superstate::loading()),
            _ => Super,
        }
    }

    #[submachine(
        machine = door::Door,
        field = door,
        entry_action = enter_loading,
        exit_action = exit_loading,
        exit_handler = door_exited,
        context = sotu::event::unit_context
    )]
    fn loading(submachine: &mut door::Door, event: &Event) -> Outcome<State> {
        match event {
            Event::Push(_) if submachine.pushes > 3 => Transition(State::idle()),
            _ => Handled,
        }
    }

    #[exit_handler]
    async fn door_exited(&mut self, exit: door::Lock) -> Outcome<State> {
        let door::Lock::Engaged = exit;

        self.entries += 1;

        Transition(State::idle())
    }

    #[action]
    async fn enter_loading(&mut self) {
        self.entries += 1;
    }

    #[action]
    fn exit_loading(submachine: &mut door::Door) {
        submachine.pushes = 0;
    }
}

fn main() {
    let _machine = Hall::default().state_machine();
}
