mod collector {
    use sotu::prelude::*;

    pub struct Push;

    #[derive(Event)]
    pub enum Event {
        Push(Push),
    }

    pub struct Collector<T> {
        pub items: Vec<T>,
    }

    #[state_machine(initial = State::idle())]
    impl<T: Default> Collector<T> {
        #[state]
        fn idle(&mut self, event: &Event) -> Outcome<State> {
            match event {
                Event::Push(_) => {
                    self.items.push(T::default());

                    Transition(State::full())
                }
            }
        }

        #[state]
        fn full(event: &Event) -> Outcome<State> {
            match event {
                Event::Push(_) => Handled,
            }
        }
    }
}

mod keeper {
    use core::marker::PhantomData;

    use sotu::prelude::*;

    pub struct Store<T>(pub T);

    #[derive(Event)]
    pub enum Event<T> {
        Store(Store<T>),
    }

    pub struct Keeper<T> {
        pub marker: PhantomData<T>,
    }

    #[state_machine(initial = State::empty(), state(derive(PartialEq, Debug)))]
    impl<T: Copy> Keeper<T> {
        #[state]
        fn empty(event: &Event<T>) -> Outcome<State<T>> {
            match event {
                Event::Store(Store(value)) => Transition(State::holding(*value)),
            }
        }

        #[state]
        fn holding(value: &mut T, event: &Event<T>) -> Outcome<State<T>> {
            match event {
                Event::Store(Store(next)) => {
                    *value = *next;

                    Handled
                }
            }
        }
    }
}

mod reader {
    use sotu::prelude::*;

    pub struct Line<'a>(pub &'a str);

    #[derive(Event)]
    pub enum Input<'a> {
        Line(Line<'a>),
    }

    pub struct Reader<'a> {
        pub lines: Vec<&'a str>,
    }

    #[state_machine(initial = State::reading())]
    impl<'a> Reader<'a> {
        #[state]
        fn reading(&mut self, event: &Input<'a>) -> Outcome<State> {
            match event {
                Input::Line(Line(text)) => {
                    self.lines.push(text);

                    Handled
                }
            }
        }
    }
}

mod buffer {
    use sotu::prelude::*;

    pub struct Fill(pub u8);

    #[derive(Event)]
    pub enum Event {
        Fill(Fill),
    }

    pub struct Buffer<const N: usize>;

    #[state_machine(initial = State::filling([0; N]), state(derive(PartialEq, Debug)))]
    impl<const N: usize> Buffer<N> {
        #[state]
        fn filling(
            slots: &mut [u8; N],
            #[default] next: &mut usize,
            event: &Event,
        ) -> Outcome<State<N>> {
            match event {
                Event::Fill(Fill(value)) => {
                    if let Some(slot) = slots.get_mut(*next) {
                        *slot = *value;
                        *next += 1;
                    }

                    if *next == N {
                        Transition(State::full(*slots))
                    } else {
                        Handled
                    }
                }
            }
        }

        #[state(local_storage(slots: [u8; N]))]
        fn full(event: &Event) -> Outcome<State<N>> {
            match event {
                Event::Fill(_) => Handled,
            }
        }
    }
}

use core::marker::PhantomData;

use sotu::prelude::*;

#[test]
fn generic_storage_with_unused_parameter() {
    use collector::{Collector, Push, State};

    let mut machine = Collector::<u8> { items: Vec::new() }.state_machine();

    let _: &State = machine.state();

    machine.handle(Push, &mut ());

    assert!(matches!(machine.state(), State::Full {}));
    assert_eq!(machine.items, [0]);
}

#[test]
fn generic_local_storage_parameter() {
    use keeper::{Keeper, State, Store};

    let mut machine = Keeper::<u8> {
        marker: PhantomData,
    }
    .state_machine();

    machine.handle(Store(7), &mut ());

    assert_eq!(machine.state(), &State::Holding { value: 7u8 });

    machine.handle(Store(9), &mut ());

    assert_eq!(machine.state(), &State::Holding { value: 9u8 });
}

#[test]
fn lifetime_in_event_type() {
    use reader::{Line, Reader};

    let text = String::from("first\nsecond");
    let mut machine = Reader { lines: Vec::new() }.state_machine();

    for line in text.lines() {
        machine.handle(Line(line), &mut ());
    }

    assert_eq!(machine.lines, ["first", "second"]);
}

#[test]
fn const_generic_local_storage() {
    use buffer::{Buffer, Fill, State};

    let mut machine = Buffer::<3>.state_machine();

    machine.handle(Fill(4), &mut ());
    machine.handle(Fill(5), &mut ());

    assert_eq!(
        machine.state(),
        &State::Filling {
            slots: [4, 5, 0],
            next: 2
        }
    );

    machine.handle(Fill(6), &mut ());

    assert_eq!(machine.state(), &State::Full { slots: [4, 5, 6] });
}
