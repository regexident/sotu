use sotu::prelude::*;

mod door {
    use sotu::prelude::*;

    pub struct Open;

    #[derive(Event)]
    pub enum Event {
        Open(Open),
    }

    pub struct Door {
        pub openings: u32,
    }

    #[state_machine(initial = State::closed(), awaitable)]
    impl Door {
        #[state(superstate = frame, entry_action = count)]
        async fn closed(event: &Event) -> Outcome<State> {
            match event {
                Event::Open(_) => Transition(State::ajar()),
            }
        }

        #[state(superstate = frame)]
        fn ajar(event: &Event) -> Outcome<State> {
            match event {
                Event::Open(_) => Super,
            }
        }

        #[superstate]
        async fn frame() -> Outcome<State> {
            Handled
        }

        #[action]
        async fn count(&mut self) {
            self.openings += 1;
        }
    }
}

#[derive(Event)]
pub enum Event {
    Open(door::Open),
}

pub struct Hall {
    pub door: Option<door::Door>,
    pub trace: Vec<&'static str>,
}

impl Hall {
    fn make_door(&mut self, _context: &mut ()) -> door::Door {
        self.trace.push("make door");

        door::Door { openings: 0 }
    }
}

#[state_machine(
    initial = Superstate::loading(),
    before_dispatch = Self::before,
    after_transition = Self::after
)]
impl Hall {
    async fn before(
        &mut self,
        _node: StateOrSuperstate<'_, State, Superstate<'_>>,
        _event: &Event,
    ) {
        self.trace.push("before");
    }

    fn after(&mut self, _source: &State, _target: &State) {
        self.trace.push("after");
    }

    #[submachine(machine = door::Door, field = door, factory = Self::make_door, superstate = region)]
    fn loading(event: &Event) -> Outcome<State> {
        match event {
            Event::Open(_) => Handled,
        }
    }

    #[superstate]
    fn region() -> Outcome<State> {
        Handled
    }
}

fn main() {
    let _machine = Hall {
        door: None,
        trace: Vec::new(),
    }
    .state_machine();
}
