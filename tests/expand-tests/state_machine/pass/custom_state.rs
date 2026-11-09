use sotu::prelude::*;

pub enum Light {
    Off {},
    On { level: u8 },
}

pub struct Lamp;

#[state_machine(initial = Light::Off {}, state(custom, name = Light))]
impl Lamp {
    #[state]
    fn off() -> Outcome<Light> {
        Transition(Light::On { level: 3 })
    }

    #[state]
    fn on(level: &mut u8) -> Outcome<Light> {
        *level += 1;

        Handled
    }
}

fn main() {
    let mut machine = Lamp.state_machine();

    machine.step(&mut ());
}
