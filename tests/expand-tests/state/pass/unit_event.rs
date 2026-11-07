use sotu::prelude::*;

#[derive(Default)]
struct Ticker {
    ticks: u32,
}

#[state_machine(initial = State::ticking())]
impl Ticker {
    #[state]
    fn ticking(&mut self) -> Outcome<State> {
        self.ticks += 1;

        Handled
    }
}

fn main() {
    let mut machine = Ticker::default().state_machine();

    machine.step(&mut ());

    assert_eq!(machine.ticks, 1);
}
