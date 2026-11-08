use sotu::prelude::*;
pub struct Sink<'ctx, 'data> {
    pub output: &'ctx mut Vec<&'data str>,
}
pub struct Writer;
impl Writer {
    fn writing(sink: &mut Sink<'_, '_>) -> Outcome<State> {
        sink.output.push("writing");
        Handled
    }
    fn enter(sink: &mut Sink<'_, '_>) {
        sink.output.push("enter");
    }
}
pub enum State {
    Writing {},
}
impl State {
    pub fn writing() -> Self {
        Self::Writing {}
    }
}
impl ::sotu::blocking::IntoStateMachine for Writer {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = Sink<'ctx, 'data>;
    type State = State;
    type Superstate<'sub> = ();
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
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Writer,
        context: &mut Sink<'ctx, 'data>,
    ) {
        match self {
            State::Writing {} => Writer::enter(context),
        }
    }
}
impl ::sotu::private::Submachine for Writer {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = Sink<'ctx, 'data>;
    type State = State;
    type Superstate<'sub> = ();
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
