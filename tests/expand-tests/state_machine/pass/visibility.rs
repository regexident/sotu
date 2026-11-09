use sotu::prelude::*;

pub(crate) struct Switch;

#[state_machine(initial = State::off(), visibility = pub(crate))]
impl Switch {
    #[state(superstate = wired)]
    fn off() -> Outcome<State> {
        Transition(State::on())
    }

    #[state(superstate = wired)]
    fn on() -> Outcome<State> {
        Super
    }

    #[superstate(initial = State::off())]
    fn wired() -> Outcome<State> {
        Handled
    }
}

fn main() {
    let mut machine = Switch.state_machine();

    machine.step(&mut ());
}
