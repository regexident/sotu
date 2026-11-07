use sotu::prelude::*;

#[derive(Default)]
pub struct Dial;

#[state_machine(initial = State::all_defaults())]
impl Dial {
    #[state]
    fn all_defaults(#[default] lit: &mut bool, #[default = 3 + 4] level: &u8) -> Outcome<State> {
        *lit = *level > 0;

        Transition(State::mixed(2, 0, 'a'))
    }

    #[state(local_storage(tail: char))]
    fn mixed(first: &u8, #[default] middle: &mut u16, last: &mut u32) -> Outcome<State> {
        *middle += u16::from(*first);
        *last += u32::from(*middle);

        Handled
    }
}

fn main() {
    let mut machine = Dial.state_machine();

    machine.step(&mut ());
}
