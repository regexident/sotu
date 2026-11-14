use sotu::prelude::*;

mod inner {
    use sotu::prelude::*;

    #[derive(Default)]
    pub struct Inner;

    #[state_machine(initial = State::leaf())]
    impl Inner {
        #[state]
        fn leaf() -> Outcome<State> {
            Handled
        }
    }
}

mod outer {
    use sotu::prelude::*;

    use super::inner;

    #[derive(Default)]
    pub struct Outer {
        inner: Option<inner::Inner>,
    }

    #[state_machine(initial = Superstate::nested())]
    impl Outer {
        #[submachine(machine = inner::Inner, field = inner)]
        fn nested() -> Outcome<State> {
            Handled
        }
    }
}

#[derive(Default)]
pub struct Nest {
    outer: Option<outer::Outer>,
}

#[state_machine(initial = Superstate::holding())]
impl Nest {
    #[submachine(machine = outer::Outer, field = outer)]
    fn holding() -> Outcome<State> {
        Handled
    }
}

fn main() {
    let mut machine = Nest::default().state_machine();

    machine.step(&mut ());
}
