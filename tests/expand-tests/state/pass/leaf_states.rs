use sotu::prelude::*;

#[derive(Default)]
struct Switch {
    flips: u32,
}

struct Flip;

struct Noop;

#[derive(Event)]
enum Event {
    Flip(Flip),
    Noop(Noop),
}

#[state_machine(initial = State::off())]
impl Switch {
    #[state]
    fn off(&mut self, event: &Event) -> Outcome<State> {
        match event {
            Event::Flip(_) => {
                self.flips += 1;

                Transition(State::on())
            }
            Event::Noop(_) => Super,
        }
    }

    #[state]
    fn on(event: &Event) -> Outcome<State> {
        match event {
            Event::Flip(_) => Transition(State::off()),
            Event::Noop(_) => Super,
        }
    }
}

fn main() {
    let mut machine = Switch::default().state_machine();

    machine.handle(Flip, &mut ());
    machine.handle(Noop, &mut ());

    assert_eq!(machine.flips, 1);
}
