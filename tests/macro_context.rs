use sotu::prelude::*;

/// A context that appends to borrowed output, holding two independent lifetimes.
pub struct Sink<'ctx, 'data> {
    output: &'ctx mut Vec<&'data str>,
    prefix: &'data str,
}

mod writer {
    use sotu::prelude::*;

    use super::Sink;

    pub struct Writer;

    pub struct Tick;

    #[derive(Event)]
    pub enum Event {
        Tick(Tick),
    }

    #[state_machine(initial = State::a(), context = Sink<'ctx, 'data>)]
    impl Writer {
        #[state(superstate = group, entry_action = enter)]
        fn a(context: &mut Sink<'_, '_>, event: &Event) -> Outcome<State> {
            match event {
                Event::Tick(_) => {
                    context.output.push("handler a");

                    Super
                }
            }
        }

        #[superstate]
        fn group(event: &Event, context: &mut Sink<'_, '_>) -> Outcome<State> {
            match event {
                Event::Tick(_) => {
                    context.output.push("superstate a");

                    Handled
                }
            }
        }

        #[action]
        fn enter(context: &mut Sink<'_, '_>) {
            context.output.push("entry a");
        }
    }
}

mod echo {
    use sotu::prelude::*;

    use super::Sink;

    pub struct Echo;

    pub struct Tick;

    #[derive(Event)]
    pub enum Event {
        Tick(Tick),
    }

    #[state_machine(initial = State::echoing(), context = Sink<'ctx, 'data>)]
    impl Echo {
        #[state]
        fn echoing(event: &Event, context: &mut Sink<'_, '_>) -> Outcome<State> {
            match event {
                Event::Tick(_) => {
                    context.output.push(context.prefix);

                    Handled
                }
            }
        }
    }
}

use echo::Echo;
use writer::{Tick, Writer};

#[test]
fn context_reaches_handlers_and_actions() {
    let mut lines = Vec::new();
    let mut sink = Sink {
        output: &mut lines,
        prefix: "a",
    };

    let mut machine = Writer.uninitialized_state_machine().init(&mut sink);

    machine.handle(Tick, &mut sink);

    assert_eq!(lines, ["entry a", "handler a", "superstate a"]);
}

#[test]
fn two_lifetime_context_is_rebuilt_per_event() {
    let mut lines = Vec::new();
    let mut machine = Echo.state_machine();

    for word in ["x", "y", "z"] {
        let mut sink = Sink {
            output: &mut lines,
            prefix: word,
        };

        machine.handle(echo::Tick, &mut sink);
    }

    assert_eq!(lines, ["x", "y", "z"]);
}
