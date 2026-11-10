use sotu::prelude::*;

pub struct Line<'a>(pub &'a str);

#[derive(Event)]
pub enum Input<'a> {
    Line(Line<'a>),
}

pub struct Reader<'a, 'b> {
    pub prefix: &'b str,
    pub lines: Vec<&'a str>,
}

#[state_machine(initial = State::waiting())]
impl<'a, 'b: 'a> Reader<'a, 'b> {
    #[state]
    fn waiting(event: &Input<'a>) -> Outcome<State<'a>> {
        match event {
            Input::Line(Line(text)) => Transition(State::remembering(*text)),
        }
    }

    #[state(superstate = reading, local_storage(last: &'a str))]
    fn remembering(event: &Input<'a>) -> Outcome<State<'a>> {
        match event {
            Input::Line(_) => Super,
        }
    }

    #[superstate]
    fn reading(&mut self, last: &mut &'a str, event: &Input<'a>) -> Outcome<State<'a>> {
        match event {
            Input::Line(Line(text)) => {
                if text.starts_with(self.prefix) {
                    self.lines.push(*last);
                }

                *last = *text;

                Handled
            }
        }
    }
}

fn main() {
    let prefix = String::from("#");
    let text = String::from("first\n#second");
    let mut machine = Reader {
        prefix: &prefix,
        lines: Vec::new(),
    }
    .state_machine();

    for line in text.lines() {
        machine.handle(Line(line), &mut ());
    }
}
