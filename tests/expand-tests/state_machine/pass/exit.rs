use sotu::prelude::*;

pub struct Finisher;

pub enum Done {
    Finished,
}

#[state_machine(initial = State::running(), exit = Done)]
impl Finisher {
    #[state(superstate = group)]
    fn running() -> Outcome<State, Done> {
        Super
    }

    #[superstate]
    fn group() -> Outcome<State, Done> {
        Exit(Done::Finished)
    }
}

fn main() {
    let mut machine = Finisher.state_machine();

    machine.step(&mut ());
}
