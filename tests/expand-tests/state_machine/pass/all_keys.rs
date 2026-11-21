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

    #[state_machine(initial = State::opened(), exit = DoorExit, state(derive(Debug)))]
    impl Door {
        #[state]
        fn opened(event: &Event) -> Outcome<State, DoorExit> {
            match event {
                Event::Close(_) => Exit(DoorExit::Closed),
            }
        }
    }
}

pub struct Sink<'ctx, 'data> {
    pub output: &'ctx mut Vec<&'data str>,
}

pub struct Start;

#[derive(Event)]
pub enum Event {
    Start(Start),
    Close(door::Close),
}

pub enum Done {
    Washed,
}

#[derive(Default)]
pub(crate) struct Washer {
    door: Option<door::Door>,
}

#[state_machine(
    initial = Mode::idle(),
    context = Sink<'ctx, 'data>,
    exit = Done,
    event_identifier = input,
    context_identifier = sink,
    submachine_identifier = child,
    exit_identifier = reason,
    before_dispatch = Self::before_dispatch,
    after_dispatch = Self::after_dispatch,
    before_transition = Self::before_transition,
    after_transition = Self::after_transition,
    visibility = pub(crate),
    state(name = Mode, derive(Debug)),
    superstate(name = Group, derive(Debug))
)]
impl Washer {
    fn before_dispatch(
        &mut self,
        _node: StateOrSuperstate<'_, Mode, Group<'_>>,
        _input: &Event,
        sink: &mut Sink<'_, '_>,
    ) {
        sink.output.push("before dispatch");
    }

    fn after_dispatch(&mut self, _node: StateOrSuperstate<'_, Mode, Group<'_>>, _input: &Event) {}

    fn before_transition(&mut self, _source: &Mode, _target: &Mode, sink: &mut Sink<'_, '_>) {
        sink.output.push("before transition");
    }

    fn after_transition(&mut self, _source: &Mode, _target: &Mode) {}

    #[state]
    fn idle(input: &Event) -> Outcome<Mode, Done> {
        match input {
            Event::Start(_) => Transition(Group::loading()),
            Event::Close(_) => Handled,
        }
    }

    #[submachine(
        machine = door::Door,
        field = door,
        context = sotu::event::unit_context,
        exit_handler = door_exited
    )]
    fn loading(child: &mut door::Door, input: &Event) -> Outcome<Mode, Done> {
        match input {
            Event::Start(_) if child.openings > 0 => Handled,
            _ => Super,
        }
    }

    #[exit_handler]
    fn door_exited(
        child: &mut door::Door,
        reason: door::DoorExit,
        sink: &mut Sink<'_, '_>,
    ) -> Outcome<Mode, Done> {
        sink.output.push("door exited");

        match reason {
            door::DoorExit::Closed if child.openings > 0 => Transition(Mode::idle()),
            door::DoorExit::Closed => Exit(Done::Washed),
        }
    }
}

fn main() {
    let mut lines = Vec::new();
    let mut sink = Sink {
        output: &mut lines,
    };

    let mut machine = Washer::default().state_machine();

    machine.handle(Start, &mut sink);
    machine.handle(door::Close, &mut sink);
}
