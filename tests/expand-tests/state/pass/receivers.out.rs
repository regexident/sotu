use sotu::prelude::*;
pub enum Event {
    Next,
}
mod mixed {
    use sotu::prelude::*;
    use super::Event;
    pub struct Mixed {
        pub visits: u32,
    }
    #[automatically_derived]
    impl ::core::default::Default for Mixed {
        #[inline]
        fn default() -> Mixed {
            Mixed {
                visits: ::core::default::Default::default(),
            }
        }
    }
    impl Mixed {
        fn exclusive(&mut self, event: &Event) -> Outcome<State> {
            self.visits += 1;
            match event {
                Event::Next => Transition(State::shared()),
            }
        }
        fn shared(&self, event: &Event) -> Outcome<State> {
            match event {
                Event::Next if self.visits > 1 => Handled,
                Event::Next => Transition(State::detached()),
            }
        }
        fn detached(event: &Event) -> Outcome<State> {
            match event {
                Event::Next => Transition(State::exclusive()),
            }
        }
    }
    pub enum State {
        Exclusive {},
        Shared {},
        Detached {},
    }
    impl State {
        pub fn exclusive() -> Self {
            Self::Exclusive {}
        }
        pub fn shared() -> Self {
            Self::Shared {}
        }
        pub fn detached() -> Self {
            Self::Detached {}
        }
    }
    impl ::sotu::blocking::IntoStateMachine for Mixed {
        type Event = Event;
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = () where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::exclusive()
        }
    }
    impl ::sotu::blocking::State<Mixed> for State {
        fn call_handler<'ctx, 'data: 'ctx>(
            &mut self,
            shared_storage: &mut Mixed,
            event: &Event,
            _context: &mut (),
        ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
            match self {
                State::Exclusive {} => Mixed::exclusive(shared_storage, event),
                State::Shared {} => Mixed::shared(shared_storage, event),
                State::Detached {} => Mixed::detached(event),
            }
        }
    }
    impl ::sotu::private::Submachine for Mixed {
        type Event = Event;
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = () where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::exclusive()
        }
    }
    impl ::sotu::private::BlockingSubmachine for Mixed {}
}
mod stateless {
    use sotu::prelude::*;
    use super::Event;
    pub struct Stateless;
    #[automatically_derived]
    impl ::core::default::Default for Stateless {
        #[inline]
        fn default() -> Stateless {
            Stateless {}
        }
    }
    impl Stateless {
        fn only(event: &Event) -> Outcome<State> {
            match event {
                Event::Next => Handled,
            }
        }
    }
    pub enum State {
        Only {},
    }
    impl State {
        pub fn only() -> Self {
            Self::Only {}
        }
    }
    impl ::sotu::blocking::IntoStateMachine for Stateless {
        type Event = Event;
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = () where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::only()
        }
    }
    impl ::sotu::blocking::State<Stateless> for State {
        fn call_handler<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Stateless,
            event: &Event,
            _context: &mut (),
        ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
            match self {
                State::Only {} => Stateless::only(event),
            }
        }
    }
    impl ::sotu::private::Submachine for Stateless {
        type Event = Event;
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = () where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::only()
        }
    }
    impl ::sotu::private::BlockingSubmachine for Stateless {}
}
fn main() {
    let mut mixed = mixed::Mixed::default().state_machine();
    let mut stateless = stateless::Stateless.state_machine();
    mixed.handle(Event::Next, &mut ());
    stateless.handle(Event::Next, &mut ());
    match (&mixed.visits, &1) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
}
