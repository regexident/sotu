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

mod lamp {
    use sotu::prelude::*;

    use super::Sink;

    pub struct Lamp;

    pub struct Toggle;

    #[derive(Event)]
    pub enum Event {
        Toggle(Toggle),
    }

    #[state_machine(
        initial = State::off(),
        context = Sink<'ctx, 'data>,
        event_identifier = input,
        context_identifier = sink
    )]
    impl Lamp {
        #[state]
        fn off(input: &Event, sink: &mut Sink<'_, '_>) -> Outcome<State> {
            match input {
                Event::Toggle(_) => {
                    sink.output.push(sink.prefix);

                    Transition(State::on())
                }
            }
        }

        #[state(entry_action = light)]
        fn on(input: &Event) -> Outcome<State> {
            match input {
                Event::Toggle(_) => Transition(State::off()),
            }
        }

        #[action]
        fn light(sink: &mut Sink<'_, '_>) {
            sink.output.push("light");
        }
    }
}

use echo::Echo;
use lamp::{Lamp, Toggle};
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

#[test]
fn renamed_event_and_context_inputs_work() {
    let mut lines = Vec::new();
    let mut sink = Sink {
        output: &mut lines,
        prefix: "toggle",
    };

    let mut machine = Lamp.state_machine();

    machine.handle(Toggle, &mut sink);

    assert!(matches!(machine.state(), lamp::State::On {}));
    assert_eq!(lines, ["toggle", "light"]);
}
