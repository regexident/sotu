use core::marker::PhantomData;

use sotu::prelude::*;

pub struct Put<T>(pub T);

#[derive(Event)]
pub enum Event<T> {
    Put(Put<T>),
}

pub struct Shelf<T, U, const N: usize> {
    pub label: U,
    pub marker: PhantomData<[T; N]>,
}

#[state_machine(initial = State::empty())]
impl<T, U, const N: usize> Shelf<T, U, N>
where
    T: Copy,
    U: Default,
{
    #[state]
    fn empty(&mut self, event: &Event<T>) -> Outcome<State<T, N>> {
        match event {
            Event::Put(Put(item)) => {
                self.label = U::default();

                Transition(State::filled([*item; N]))
            }
        }
    }

    #[state(superstate = stocked, local_storage(items: [T; N]))]
    fn filled(event: &Event<T>) -> Outcome<State<T, N>> {
        match event {
            Event::Put(_) => Super,
        }
    }

    #[superstate]
    fn stocked(items: &mut [T; N], event: &Event<T>) -> Outcome<State<T, N>> {
        match event {
            Event::Put(Put(item)) => {
                if let Some(first) = items.first_mut() {
                    *first = *item;
                }

                Handled
            }
        }
    }
}

fn main() {
    let mut machine = Shelf::<u8, String, 2> {
        label: String::new(),
        marker: PhantomData,
    }
    .state_machine();

    machine.handle(Put(3u8), &mut ());
    machine.handle(Put(4u8), &mut ());
}
