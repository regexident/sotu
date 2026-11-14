use sotu::prelude::*;
mod door {
    use sotu::prelude::*;
    pub struct Door;
    #[automatically_derived]
    impl ::core::default::Default for Door {
        #[inline]
        fn default() -> Door {
            Door {}
        }
    }
    impl Door {
        fn closed() -> Outcome<State> {
            Handled
        }
    }
    pub enum State {
        Closed {},
    }
    impl State {
        pub fn closed() -> Self {
            Self::Closed {}
        }
    }
    impl ::sotu::blocking::IntoStateMachine for Door {
        type Event = ();
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = () where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::closed()
        }
    }
    impl ::sotu::blocking::State<Door> for State {
        fn call_handler<'ctx, 'data: 'ctx>(
            &mut self,
            _shared_storage: &mut Door,
            _event: &(),
            _context: &mut (),
        ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
            match self {
                State::Closed {} => Door::closed(),
            }
        }
    }
    impl ::sotu::private::Submachine for Door {
        type Event = ();
        type Context<'ctx, 'data: 'ctx> = ();
        type State = State;
        type Superstate<'sub> = () where Self: 'sub;
        type Exit = ::core::convert::Infallible;
        fn initial() -> State {
            State::closed()
        }
    }
    impl ::sotu::private::BlockingSubmachine for Door {}
    impl ::sotu::private::ExitlessSubmachine for Door {}
}
pub struct House {
    door: Option<door::Door>,
}
#[automatically_derived]
impl ::core::default::Default for House {
    #[inline]
    fn default() -> House {
        House {
            door: ::core::default::Default::default(),
        }
    }
}
fn main() {}
