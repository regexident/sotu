use sotu::prelude::*;

pub struct Sink<'ctx, 'data> {
    pub output: &'ctx mut Vec<&'data str>,
}

pub struct Writer;

#[state_machine(
    initial = State::writing(),
    context = Sink<'ctx, 'data>,
    context_identifier = sink
)]
impl Writer {
    #[state(entry_action = enter)]
    fn writing(sink: &mut Sink<'_, '_>) -> Outcome<State> {
        sink.output.push("writing");

        Handled
    }

    #[action]
    fn enter(sink: &mut Sink<'_, '_>) {
        sink.output.push("enter");
    }
}

fn main() {
    let mut lines = Vec::new();
    let mut sink = Sink {
        output: &mut lines,
    };

    let mut machine = Writer.state_machine();

    machine.step(&mut sink);
}
