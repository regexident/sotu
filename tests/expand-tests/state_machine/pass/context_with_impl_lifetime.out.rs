use sotu::prelude::*;
pub struct Tagged<'ctx, 'data, 'a> {
    pub output: &'ctx mut Vec<&'data str>,
    pub tag: &'a str,
}
pub struct Tagger<'a> {
    pub tag: &'a str,
}
impl<'a> Tagger<'a> {
    fn tagging(&mut self, context: &mut Tagged<'_, '_, 'a>) -> Outcome<State> {
        if context.tag == self.tag {
            context.output.push("match");
        }
        Handled
    }
}
pub enum State {
    Tagging {},
}
impl State {
    pub fn tagging() -> Self {
        Self::Tagging {}
    }
}
impl<'a> ::sotu::blocking::IntoStateMachine for Tagger<'a> {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = Tagged<'ctx, 'data, 'a>;
    type State = State;
    type Superstate<'sub> = ();
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::tagging()
    }
}
impl<'a> ::sotu::blocking::State<Tagger<'a>> for State {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Tagger<'a>,
        _event: &(),
        context: &mut Tagged<'ctx, 'data, 'a>,
    ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
        match self {
            State::Tagging {} => Tagger::<'a>::tagging(shared_storage, context),
        }
    }
}
impl<'a> ::sotu::private::Submachine for Tagger<'a> {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = Tagged<'ctx, 'data, 'a>;
    type State = State;
    type Superstate<'sub> = ();
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::tagging()
    }
}
impl<'a> ::sotu::private::BlockingSubmachine for Tagger<'a> {}
fn main() {
    let tag = String::from("tag");
    let mut lines = Vec::new();
    let mut context = Tagged {
        output: &mut lines,
        tag: &tag,
    };
    let mut machine = Tagger { tag: &tag }.state_machine();
    machine.step(&mut context);
}
