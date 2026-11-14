use core::marker::PhantomData;
use sotu::prelude::*;
pub struct Put<T>(pub T);
pub enum Event<T> {
    Put(Put<T>),
}
impl<T> ::core::convert::From<Put<T>> for Event<T> {
    fn from(value: Put<T>) -> Self {
        Self::Put(value)
    }
}
impl<T> ::sotu::event::Variant<Put<T>> for Event<T> {
    fn from_variant(value: Put<T>) -> Self {
        Self::Put(value)
    }
    fn into_variant(self) -> ::core::result::Result<Put<T>, Self> {
        match self {
            Self::Put(value) => ::core::result::Result::Ok(value),
        }
    }
    fn as_variant_ref(&self) -> ::core::option::Option<&Put<T>> {
        match self {
            Self::Put(value) => ::core::option::Option::Some(value),
        }
    }
}
impl<SotuParentEvent, T> ::sotu::event::SubEvent<SotuParentEvent> for Event<T>
where
    SotuParentEvent: ::sotu::event::Variant<Put<T>>,
{
    fn accepts(parent: &SotuParentEvent) -> bool {
        <SotuParentEvent as ::sotu::event::Variant<Put<T>>>::as_variant_ref(parent)
            .is_some()
    }
    fn project(
        parent: SotuParentEvent,
    ) -> ::core::result::Result<Self, SotuParentEvent> {
        let parent = match <SotuParentEvent as ::sotu::event::Variant<
            Put<T>,
        >>::into_variant(parent) {
            ::core::result::Result::Ok(value) => {
                return ::core::result::Result::Ok(Self::Put(value));
            }
            ::core::result::Result::Err(parent) => parent,
        };
        ::core::result::Result::Err(parent)
    }
    fn embed(self) -> SotuParentEvent {
        match self {
            Self::Put(value) => {
                <SotuParentEvent as ::sotu::event::Variant<Put<T>>>::from_variant(value)
            }
        }
    }
}
pub struct Shelf<T, U, const N: usize> {
    pub label: U,
    pub marker: PhantomData<[T; N]>,
}
impl<T, U, const N: usize> Shelf<T, U, N>
where
    T: Copy,
    U: Default,
{
    fn empty(&mut self, event: &Event<T>) -> Outcome<State<T, N>> {
        match event {
            Event::Put(Put(item)) => {
                self.label = U::default();
                Transition(State::filled([*item; N]))
            }
        }
    }
    fn filled(event: &Event<T>) -> Outcome<State<T, N>> {
        match event {
            Event::Put(_) => Super,
        }
    }
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
pub enum State<T, const N: usize>
where
    T: Copy,
{
    Empty {},
    Filled { items: [T; N] },
}
impl<T, const N: usize> State<T, N>
where
    T: Copy,
{
    pub fn empty() -> Self {
        Self::Empty {}
    }
    pub fn filled(items: [T; N]) -> Self {
        Self::Filled { items }
    }
}
pub enum Superstate<'sub, T, const N: usize>
where
    T: Copy,
{
    Stocked { items: &'sub mut [T; N] },
}
impl<T, U, const N: usize> ::sotu::blocking::IntoStateMachine for Shelf<T, U, N>
where
    T: Copy,
    U: Default,
{
    type Event = Event<T>;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State<T, N>;
    type Superstate<'sub> = Superstate<'sub, T, N> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State<T, N> {
        State::empty()
    }
}
impl<T, U, const N: usize> ::sotu::blocking::State<Shelf<T, U, N>> for State<T, N>
where
    T: Copy,
    U: Default,
{
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Shelf<T, U, N>,
        event: &Event<T>,
        _context: &mut (),
    ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
        match self {
            State::Empty {} => Shelf::<T, U, N>::empty(shared_storage, event),
            State::Filled { .. } => Shelf::<T, U, N>::filled(event),
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate<'_, T, N>> {
        match self {
            State::Empty {} => ::core::option::Option::None,
            State::Filled { items } => {
                ::core::option::Option::Some(Superstate::Stocked { items })
            }
        }
    }
}
impl<'sub, T, U, const N: usize> ::sotu::blocking::Superstate<Shelf<T, U, N>>
for Superstate<'sub, T, N>
where
    T: Copy,
    U: Default,
{
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Shelf<T, U, N>,
        event: &Event<T>,
        _context: &mut (),
    ) -> ::sotu::Outcome<State<T, N>, ::core::convert::Infallible> {
        match self {
            Superstate::Stocked { items } => Shelf::<T, U, N>::stocked(items, event),
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate<'_, T, N>> {
        match self {
            Superstate::Stocked { .. } => ::core::option::Option::None,
        }
    }
    fn same_state(&self, other: &Superstate<'_, T, N>) -> bool {
        match (self, other) {
            (Superstate::Stocked { .. }, Superstate::Stocked { .. }) => true,
        }
    }
}
impl<T, U, const N: usize> ::sotu::private::Submachine for Shelf<T, U, N>
where
    T: Copy,
    U: Default,
{
    type Event = Event<T>;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State<T, N>;
    type Superstate<'sub> = Superstate<'sub, T, N> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State<T, N> {
        State::empty()
    }
}
impl<T, U, const N: usize> ::sotu::private::BlockingSubmachine for Shelf<T, U, N>
where
    T: Copy,
    U: Default,
{}
impl<T, U, const N: usize> ::sotu::private::ExitlessSubmachine for Shelf<T, U, N>
where
    T: Copy,
    U: Default,
{}
fn main() {
    let mut machine = Shelf::<u8, String, 2> {
        label: String::new(),
        marker: PhantomData,
    }
        .state_machine();
    machine.handle(Put(3u8), &mut ());
    machine.handle(Put(4u8), &mut ());
}
