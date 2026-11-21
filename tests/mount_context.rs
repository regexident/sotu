mod common;

use sotu::prelude::*;

// MARK: Machines

mod lamp {
    use sotu::prelude::*;

    /// A child without a context.
    #[derive(Default)]
    pub struct Lamp {
        pub toggles: u32,
    }

    pub struct Toggle;

    #[derive(Event)]
    pub enum Event {
        Toggle(Toggle),
    }

    #[state_machine(initial = State::off())]
    impl Lamp {
        #[state]
        fn off(&mut self, event: &Event) -> Outcome<State> {
            match event {
                Event::Toggle(_) => {
                    self.toggles += 1;

                    Handled
                }
            }
        }
    }
}

mod room {
    use sotu::prelude::*;

    use super::lamp;

    /// A parent with a `u32` context that mounts a child without one.
    #[derive(Default)]
    pub struct Room {
        pub lamp: lamp::Lamp,
    }

    pub struct Knock;

    #[derive(Event)]
    pub enum Event {
        Toggle(lamp::Toggle),
        Knock(Knock),
    }

    #[state_machine(initial = Superstate::lit(), context = u32)]
    impl Room {
        #[submachine(machine = lamp::Lamp, field = lamp, context = sotu::event::unit_context)]
        fn lit(event: &Event, context: &mut u32) -> Outcome<State> {
            match event {
                Event::Toggle(_) => Handled,
                Event::Knock(_) => {
                    *context += 1;

                    Handled
                }
            }
        }
    }
}

// MARK: Tests

#[test]
fn unit_context_mounts_unit_context_child() {
    let mut context = 0;
    let mut machine = room::Room::default()
        .uninitialized_state_machine()
        .init(&mut context);

    machine.handle(room::Event::Toggle(lamp::Toggle), &mut context);
    machine.handle(room::Event::Knock(room::Knock), &mut context);

    assert_eq!(machine.lamp.toggles, 1);
    assert_eq!(context, 1);
}
