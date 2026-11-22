use sotu::prelude::*;

pub enum Event {
    Tick,
    Reset,
}

#[derive(Default)]
pub struct Blinker {
    pub ticks: u32,
}

#[state_machine(initial = State::on(0))]
impl Blinker {
    #[superstate]
    async fn outer(&mut self, counter: &u32, event: &Event) -> Outcome<State> {
        match event {
            Event::Tick => {
                self.ticks = *counter;

                Handled
            }
            Event::Reset => Transition(State::off()),
        }
    }

    #[superstate(superstate = outer)]
    async fn blinking(counter: &mut u32, event: &Event) -> Outcome<State> {
        match event {
            Event::Tick => {
                *counter += 1;

                Super
            }
            Event::Reset => Super,
        }
    }

    #[state(superstate = blinking)]
    async fn on(counter: &mut u32, event: &Event) -> Outcome<State> {
        match event {
            Event::Tick if *counter > 3 => Transition(State::off()),
            Event::Tick | Event::Reset => Super,
        }
    }

    #[state]
    fn off(event: &Event) -> Outcome<State> {
        match event {
            Event::Tick => Handled,
            Event::Reset => Transition(State::on(0)),
        }
    }
}

fn main() {
    let _machine = Blinker::default().state_machine();
}
