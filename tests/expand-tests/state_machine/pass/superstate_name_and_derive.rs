use sotu::prelude::*;

pub struct Blinker;

#[state_machine(
    initial = State::on(0),
    superstate(name = Group, derive(Debug))
)]
impl Blinker {
    #[state(superstate = blinking)]
    fn on(count: &mut u8) -> Outcome<State> {
        if *count > 9 {
            Transition(State::on(0))
        } else {
            Super
        }
    }

    #[superstate(initial = State::on(0))]
    fn blinking(count: &mut u8) -> Outcome<State> {
        *count += 1;

        Handled
    }
}

fn main() {
    let mut machine = Blinker.state_machine();

    machine.step(&mut ());
}
