use sotu::prelude::*;
pub struct Sink<'ctx, 'data> {
    pub output: &'ctx mut Vec<&'data str>,
}
pub struct Writer;
impl Writer {
    fn writing(context: &mut Sink<'_, '_>) -> Outcome<State> {
        context.output.push("writing");
        Super
    }
    fn idle() -> Outcome<State> {
        Handled
    }
    fn group(&mut self, context: &mut Sink<'_, '_>) -> Outcome<State> {
        context.output.push("group");
        Transition(State::idle())
    }
    fn enter(context: &mut Sink<'_, '_>) {
        context.output.push("enter");
    }
    fn leave(&mut self) {}
}
pub enum State {
    Writing {},
    Idle {},
}
impl State {
    pub fn writing() -> Self {
        Self::Writing {}
    }
    pub fn idle() -> Self {
        Self::Idle {}
    }
}
pub enum Superstate {
    Group {},
}
impl ::sotu::blocking::IntoStateMachine for Writer {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = Sink<'ctx, 'data>;
    type State = State;
    type Superstate<'sub> = Superstate where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::writing()
    }
}
impl ::sotu::blocking::State<Writer> for State {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Writer,
        _event: &(),
        context: &mut Sink<'ctx, 'data>,
    ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
        match self {
            State::Writing {} => Writer::writing(context),
            State::Idle {} => Writer::idle(),
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Writer,
        context: &mut Sink<'ctx, 'data>,
    ) {
        match self {
            State::Writing {} => Writer::enter(context),
            State::Idle {} => {}
        }
    }
    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Writer,
        _context: &mut Sink<'ctx, 'data>,
    ) {
        match self {
            State::Writing {} => Writer::leave(shared_storage),
            State::Idle {} => {}
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate> {
        match self {
            State::Writing {} => ::core::option::Option::Some(Superstate::Group {}),
            State::Idle {} => ::core::option::Option::None,
        }
    }
}
impl ::sotu::blocking::Superstate<Writer> for Superstate {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Writer,
        _event: &(),
        context: &mut Sink<'ctx, 'data>,
    ) -> ::sotu::Outcome<State, ::core::convert::Infallible> {
        match self {
            Superstate::Group {} => Writer::group(shared_storage, context),
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate> {
        match self {
            Superstate::Group {} => ::core::option::Option::None,
        }
    }
    fn same_state(&self, other: &Superstate) -> bool {
        match (self, other) {
            (Superstate::Group { .. }, Superstate::Group { .. }) => true,
        }
    }
}
impl ::sotu::private::Submachine for Writer {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = Sink<'ctx, 'data>;
    type State = State;
    type Superstate<'sub> = Superstate where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::writing()
    }
}
impl ::sotu::private::BlockingSubmachine for Writer {}
fn main() {
    let mut lines = Vec::new();
    let mut sink = Sink { output: &mut lines };
    let mut machine = Writer.state_machine();
    machine.step(&mut sink);
}
