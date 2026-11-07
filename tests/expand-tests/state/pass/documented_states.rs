use sotu::prelude::*;

#[derive(Default)]
struct Door {
    openings: u32,
}

enum Event {
    Toggle,
}

#[state_machine(initial = State::door_closed())]
impl Door {
    /// The door is closed.
    ///
    /// Toggling it opens it.
    #[state]
    #[inline]
    fn door_closed(&mut self, event: &Event) -> Outcome<State> {
        self.count_opening();

        match event {
            Event::Toggle => Transition(State::door_open()),
        }
    }

    /// The door is open.
    #[state()]
    fn door_open(event: &Event) -> Outcome<State> {
        match event {
            Event::Toggle => Transition(State::door_closed()),
        }
    }

    /// Count one opening of the door.
    fn count_opening(&mut self) {
        self.openings += 1;
    }
}

fn main() {
    let mut machine = Door::default().state_machine();

    machine.handle(Event::Toggle, &mut ());

    assert_eq!(machine.openings, 1);
}
