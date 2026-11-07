use sotu::prelude::*;

pub enum Event {
    Next,
}

#[derive(Default)]
pub struct Meter {
    pub readings: u32,
}

#[state_machine(initial = State::counting(0, 1, false))]
impl Meter {
    /// The meter counts.
    #[state(local_storage(started: bool))]
    fn counting(&mut self, counter: &mut u32, step: &u32, event: &Event) -> Outcome<State> {
        match event {
            Event::Next => {
                *counter += *step;
                self.readings = *counter;

                Transition(State::holding(*counter, 0))
            }
        }
    }

    /// The meter holds a value it does not read while handling events.
    #[state(local_storage(held: u32, label: u8))]
    fn holding(event: &Event) -> Outcome<State> {
        match event {
            Event::Next => Transition(State::shadowing(0)),
        }
    }

    /// A field may share its name with a parameter of the generated methods.
    #[state]
    fn shadowing(&mut self, shared_storage: &mut u32) -> Outcome<State> {
        self.readings = *shared_storage;

        Handled
    }
}

fn main() {
    let mut machine = Meter::default().state_machine();

    machine.handle(Event::Next, &mut ());

    assert_eq!(machine.readings, 1);
}
