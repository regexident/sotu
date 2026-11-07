use sotu::prelude::*;

#[derive(Default)]
pub struct Gate;

#[state_machine(initial = State::door_open(0))]
impl Gate {
    /// The gate stands open.
    #[state(name = DoorOpen)]
    fn waiting(ticks: &mut u32) -> Outcome<State> {
        *ticks += 1;

        Transition(State::closed())
    }

    #[state(name = Closed)]
    fn shut() -> Outcome<State> {
        Handled
    }
}

fn main() {
    let mut machine = Gate.state_machine();

    machine.step(&mut ());
}
