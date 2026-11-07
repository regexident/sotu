use sotu::prelude::*;

pub enum Event {
    Next,
}

mod mixed {
    use sotu::prelude::*;

    use super::Event;

    #[derive(Default)]
    pub struct Mixed {
        pub visits: u32,
    }

    #[state_machine(initial = State::exclusive())]
    impl Mixed {
        #[state]
        fn exclusive(&mut self, event: &Event) -> Outcome<State> {
            self.visits += 1;

            match event {
                Event::Next => Transition(State::shared()),
            }
        }

        #[state]
        fn shared(&self, event: &Event) -> Outcome<State> {
            match event {
                Event::Next if self.visits > 1 => Handled,
                Event::Next => Transition(State::detached()),
            }
        }

        #[state]
        fn detached(event: &Event) -> Outcome<State> {
            match event {
                Event::Next => Transition(State::exclusive()),
            }
        }
    }
}

mod stateless {
    use sotu::prelude::*;

    use super::Event;

    #[derive(Default)]
    pub struct Stateless;

    #[state_machine(initial = State::only())]
    impl Stateless {
        #[state]
        fn only(event: &Event) -> Outcome<State> {
            match event {
                Event::Next => Handled,
            }
        }
    }
}

fn main() {
    let mut mixed = mixed::Mixed::default().state_machine();
    let mut stateless = stateless::Stateless.state_machine();

    mixed.handle(Event::Next, &mut ());
    stateless.handle(Event::Next, &mut ());

    assert_eq!(mixed.visits, 1);
}
