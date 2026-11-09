use sotu::prelude::*;

pub struct Counter;

#[state_machine(
    initial = Phase::counting(0),
    state(name = Phase, derive(Clone, Debug, PartialEq))
)]
impl Counter {
    #[state]
    fn counting(count: &mut u8) -> Outcome<Phase> {
        *count += 1;

        Handled
    }
}

fn main() {
    let mut machine = Counter.state_machine();

    machine.step(&mut ());
}
