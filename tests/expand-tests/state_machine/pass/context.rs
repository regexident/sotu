use sotu::prelude::*;

pub struct Sink<'ctx, 'data> {
    pub output: &'ctx mut Vec<&'data str>,
}

pub struct Writer;

#[state_machine(initial = State::writing(), context = Sink<'ctx, 'data>)]
impl Writer {
    #[state(superstate = group, entry_action = enter, exit_action = leave)]
    fn writing(context: &mut Sink<'_, '_>) -> Outcome<State> {
        context.output.push("writing");

        Super
    }

    #[state]
    fn idle() -> Outcome<State> {
        Handled
    }

    #[superstate]
    fn group(&mut self, context: &mut Sink<'_, '_>) -> Outcome<State> {
        context.output.push("group");

        Transition(State::idle())
    }

    #[action]
    fn enter(context: &mut Sink<'_, '_>) {
        context.output.push("enter");
    }

    #[action]
    fn leave(&mut self) {}
}

fn main() {
    let mut lines = Vec::new();
    let mut sink = Sink {
        output: &mut lines,
    };

    let mut machine = Writer.state_machine();

    machine.step(&mut sink);
}
