use sotu::prelude::*;

mod finisher {
    use sotu::prelude::*;

    pub struct Finisher;

    pub struct Finish;

    #[derive(Event)]
    pub enum Event {
        Finish(Finish),
    }

    pub enum Done {
        Finished,
    }

    #[state_machine(initial = State::running(), exit = Done)]
    impl Finisher {
        #[state]
        fn running(event: &Event) -> Outcome<State, Done> {
            match event {
                Event::Finish(_) => Exit(Done::Finished),
            }
        }
    }
}

use finisher::{Done, Finish, Finisher};

#[test]
fn exit_terminates_macro_machine() {
    let mut machine = Finisher.state_machine();

    assert!(matches!(
        machine.handle(Finish, &mut ()),
        Some(&Done::Finished)
    ));
    assert!(machine.is_terminated());
}
