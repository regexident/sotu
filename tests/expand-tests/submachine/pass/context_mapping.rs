use sotu::prelude::*;

mod printer {
    use sotu::prelude::*;

    /// The context of the printer, borrowed from the parent's context.
    pub struct Sink<'ctx, 'data> {
        pub lines: &'ctx mut Vec<&'data str>,
    }

    #[derive(Default)]
    pub struct Printer;

    #[state_machine(
        initial = State::idle(),
        context = Sink<'ctx, 'data>,
        before_dispatch = Self::before,
        after_transition = Self::after
    )]
    impl Printer {
        fn before(
            &mut self,
            _node: StateOrSuperstate<'_, State, Superstate>,
            _event: &(),
            context: &mut Sink<'_, '_>,
        ) {
            context.lines.push("dispatch");
        }

        fn after(&mut self, _source: &State, _target: &State, context: &mut Sink<'_, '_>) {
            context.lines.push("transition");
        }

        #[state(superstate = powered, entry_action = warm)]
        fn idle() -> Outcome<State> {
            Transition(State::printing())
        }

        #[state(superstate = powered)]
        fn printing(context: &mut Sink<'_, '_>) -> Outcome<State> {
            context.lines.push("page");

            Super
        }

        #[superstate]
        fn powered() -> Outcome<State> {
            Handled
        }

        #[action]
        fn warm(context: &mut Sink<'_, '_>) {
            context.lines.push("warm");
        }
    }
}

/// The context of the office, which the printer borrows for each callback.
pub struct Writer<'ctx, 'data> {
    lines: &'ctx mut Vec<&'data str>,
}

fn map_context<'a, 'ctx, 'data>(writer: &'a mut Writer<'ctx, 'data>) -> printer::Sink<'a, 'data> {
    printer::Sink {
        lines: &mut *writer.lines,
    }
}

#[derive(Default)]
pub struct Office {
    printer: printer::Printer,
}

#[state_machine(initial = Superstate::printing(), context = Writer<'ctx, 'data>)]
impl Office {
    #[submachine(machine = printer::Printer, field = printer, context = map_context)]
    fn printing() -> Outcome<State> {
        Handled
    }
}

fn main() {
    let mut lines = Vec::new();
    let mut machine = Office::default().state_machine();

    for _ in 0..2 {
        let mut writer = Writer { lines: &mut lines };

        machine.step(&mut writer);
    }
}
