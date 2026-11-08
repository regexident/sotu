use sotu::prelude::*;

mod borrowing {
    use sotu::prelude::*;

    #[derive(Default)]
    pub struct Blinker;

    #[state_machine(initial = Superstate::blinking())]
    impl Blinker {
        #[state(superstate = blinking, local_storage(counter: u32))]
        fn on() -> Outcome<State> {
            Super
        }

        #[superstate(initial = State::on(0))]
        fn blinking(counter: &mut u32) -> Outcome<State> {
            *counter += 1;

            Handled
        }

        #[superstate(superstate = blinking)]
        fn without_initial(counter: &mut u32) -> Outcome<State> {
            if *counter > 0 { Handled } else { Super }
        }
    }
}

mod field_less {
    use sotu::prelude::*;

    #[derive(Default)]
    pub struct Lamp;

    #[state_machine(initial = Superstate::group())]
    impl Lamp {
        #[state(superstate = group)]
        fn member() -> Outcome<State> {
            Super
        }

        #[superstate(initial = State::member())]
        fn group() -> Outcome<State> {
            Handled
        }
    }
}

fn main() {
    let mut blinker = borrowing::Blinker.state_machine();
    let mut lamp = field_less::Lamp.state_machine();

    blinker.step(&mut ());
    lamp.step(&mut ());
}
