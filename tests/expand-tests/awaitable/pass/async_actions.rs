use sotu::prelude::*;

pub enum Event {
    Flip,
}

#[derive(Default)]
pub struct Lamp {
    pub flips: u32,
}

#[state_machine(initial = State::off())]
impl Lamp {
    #[superstate(entry_action = power_up, exit_action = power_down)]
    fn powered() -> Outcome<State> {
        Super
    }

    #[state(superstate = powered, entry_action = light, exit_action = dim)]
    fn on(lit: &mut bool, event: &Event) -> Outcome<State> {
        match event {
            Event::Flip if *lit => Transition(State::off()),
            Event::Flip => Handled,
        }
    }

    #[state]
    fn off(event: &Event) -> Outcome<State> {
        match event {
            Event::Flip => Transition(State::on(false)),
        }
    }

    #[action]
    async fn power_up(&mut self) {
        self.flips += 1;
    }

    #[action]
    fn power_down(&mut self) {
        self.flips += 1;
    }

    #[action]
    async fn light(&mut self, lit: &mut bool) {
        *lit = true;
        self.flips += 1;
    }

    #[action]
    fn dim(lit: &mut bool) {
        *lit = false;
    }
}

fn main() {
    let _machine = Lamp::default().state_machine();
}
