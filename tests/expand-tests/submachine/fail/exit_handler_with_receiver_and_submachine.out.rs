use sotu::prelude::*;
mod door {
    use sotu::prelude::*;
    pub enum DoorExit {
        Closed,
    }
    pub struct Door {
        pub openings: u32,
    }
    #[automatically_derived]
    impl ::core::default::Default for Door {
        #[inline]
        fn default() -> Door {
            Door {
                openings: ::core::default::Default::default(),
            }
        }
    }
    impl Door {
        fn closed() -> Outcome<State, DoorExit> {
            Exit(DoorExit::Closed)
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
        type Exit = DoorExit;
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
        ) -> ::sotu::Outcome<Self, DoorExit> {
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
        type Exit = DoorExit;
        fn initial() -> State {
            State::closed()
        }
    }
    impl ::sotu::private::BlockingSubmachine for Door {}
}
pub struct House {
    visits: u32,
    door: Option<door::Door>,
}
#[automatically_derived]
impl ::core::default::Default for House {
    #[inline]
    fn default() -> House {
        House {
            visits: ::core::default::Default::default(),
            door: ::core::default::Default::default(),
        }
    }
}
fn main() {}
