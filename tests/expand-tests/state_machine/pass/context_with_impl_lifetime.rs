use sotu::prelude::*;

pub struct Tagged<'ctx, 'data, 'a> {
    pub output: &'ctx mut Vec<&'data str>,
    pub tag: &'a str,
}

pub struct Tagger<'a> {
    pub tag: &'a str,
}

#[state_machine(initial = State::tagging(), context = Tagged<'ctx, 'data, 'a>)]
impl<'a> Tagger<'a> {
    #[state]
    fn tagging(&mut self, context: &mut Tagged<'_, '_, 'a>) -> Outcome<State> {
        if context.tag == self.tag {
            context.output.push("match");
        }

        Handled
    }
}

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
