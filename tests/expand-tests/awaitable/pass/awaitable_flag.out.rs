use sotu::prelude::*;
pub enum Event {
    Press,
    Finish,
}
pub enum Done {
    Finished,
}
pub struct Button {
    pub presses: u32,
}
#[automatically_derived]
impl ::core::default::Default for Button {
    #[inline]
    fn default() -> Button {
        Button {
            presses: ::core::default::Default::default(),
        }
    }
}
impl Button {
    fn idle(event: &Event) -> Outcome<State, Done> {
        match event {
            Event::Press => Transition(State::idle()),
            Event::Finish => Exit(Done::Finished),
        }
    }
    fn count(&mut self) {
        self.presses += 1;
    }
}
pub enum State {
    Idle {},
}
impl State {
    pub fn idle() -> Self {
        Self::Idle {}
    }
}
impl ::sotu::awaitable::IntoStateMachine for Button {
    type Event = Event;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = () where Self: 'sub;
    type Exit = Done;
    fn initial() -> State {
        State::idle()
    }
}
impl ::sotu::awaitable::State<Button> for State {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Button,
        event: &Event,
        _context: &mut (),
    ) -> impl ::core::future::Future<Output = ::sotu::Outcome<Self, Done>> {
        async move {
            match self {
                State::Idle {} => Button::idle(event),
            }
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Button,
        _context: &mut (),
    ) -> impl ::core::future::Future<Output = ()> {
        async move {
            match self {
                State::Idle {} => Button::count(shared_storage),
            }
        }
    }
}
impl ::sotu::private::Submachine for Button {
    type Event = Event;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = () where Self: 'sub;
    type Exit = Done;
    fn initial() -> State {
        State::idle()
    }
}
fn main() {
    let _machine = Button::default().state_machine();
}
