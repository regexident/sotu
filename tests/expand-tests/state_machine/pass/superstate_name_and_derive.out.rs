use sotu::prelude::*;
pub struct Blinker;
impl Blinker {
    fn on(count: &mut u8) -> Outcome<State> {
        if *count > 9 { Transition(State::on(0)) } else { Super }
    }
    fn blinking(count: &mut u8) -> Outcome<State> {
        *count += 1;
        Handled
    }
}
pub enum State {
    On { count: u8 },
}
impl State {
    pub fn on(count: u8) -> Self {
        Self::On { count }
    }
}
pub enum Group<'sub> {
    Blinking { count: &'sub mut u8 },
}
#[automatically_derived]
impl<'sub> ::core::fmt::Debug for Group<'sub> {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            Group::Blinking { count: __self_0 } => {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "Blinking",
                    "count",
                    &__self_0,
                )
            }
        }
    }
}
impl Group<'_> {
    pub fn blinking() -> State {
        State::on(0)
    }
}
impl ::sotu::blocking::IntoStateMachine for Blinker {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Group<'sub> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::on(0)
    }
}
impl ::sotu::blocking::State<Blinker> for State {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Blinker,
        _event: &(),
        _context: &mut (),
    ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
        match self {
            State::On { count } => Blinker::on(count),
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Group<'_>> {
        match self {
            State::On { count } => {
                ::core::option::Option::Some(Group::Blinking { count })
            }
        }
    }
}
impl<'sub> ::sotu::blocking::Superstate<Blinker> for Group<'sub> {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Blinker,
        _event: &(),
        _context: &mut (),
    ) -> ::sotu::Outcome<State, ::core::convert::Infallible> {
        match self {
            Group::Blinking { count } => Blinker::blinking(count),
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Group<'_>> {
        match self {
            Group::Blinking { .. } => ::core::option::Option::None,
        }
    }
    fn same_state(&self, other: &Group<'_>) -> bool {
        match (self, other) {
            (Group::Blinking { .. }, Group::Blinking { .. }) => true,
        }
    }
}
impl ::sotu::private::Submachine for Blinker {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Group<'sub> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::on(0)
    }
}
impl ::sotu::private::BlockingSubmachine for Blinker {}
impl ::sotu::private::ExitlessSubmachine for Blinker {}
fn main() {
    let mut machine = Blinker.state_machine();
    machine.step(&mut ());
}
