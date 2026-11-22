use sotu::prelude::*;
pub enum Event {
    Tick,
    Reset,
}
pub struct Blinker {
    pub ticks: u32,
}
#[automatically_derived]
impl ::core::default::Default for Blinker {
    #[inline]
    fn default() -> Blinker {
        Blinker {
            ticks: ::core::default::Default::default(),
        }
    }
}
impl Blinker {
    async fn outer(&mut self, counter: &u32, event: &Event) -> Outcome<State> {
        match event {
            Event::Tick => {
                self.ticks = *counter;
                Handled
            }
            Event::Reset => Transition(State::off()),
        }
    }
    async fn blinking(counter: &mut u32, event: &Event) -> Outcome<State> {
        match event {
            Event::Tick => {
                *counter += 1;
                Super
            }
            Event::Reset => Super,
        }
    }
    async fn on(counter: &mut u32, event: &Event) -> Outcome<State> {
        match event {
            Event::Tick if *counter > 3 => Transition(State::off()),
            Event::Tick | Event::Reset => Super,
        }
    }
    fn off(event: &Event) -> Outcome<State> {
        match event {
            Event::Tick => Handled,
            Event::Reset => Transition(State::on(0)),
        }
    }
}
pub enum State {
    On { counter: u32 },
    Off {},
}
impl State {
    pub fn on(counter: u32) -> Self {
        Self::On { counter }
    }
    pub fn off() -> Self {
        Self::Off {}
    }
}
pub enum Superstate<'sub> {
    Outer { counter: &'sub u32 },
    Blinking { counter: &'sub mut u32 },
}
impl ::sotu::awaitable::IntoStateMachine for Blinker {
    type Event = Event;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::on(0)
    }
}
impl ::sotu::awaitable::State<Blinker> for State {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Blinker,
        event: &Event,
        _context: &mut (),
    ) -> impl ::core::future::Future<
        Output = ::sotu::Outcome<Self, ::core::convert::Infallible>,
    > {
        async move {
            match self {
                State::On { counter } => Blinker::on(counter, event).await,
                State::Off {} => Blinker::off(event),
            }
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate<'_>> {
        match self {
            State::On { counter } => {
                ::core::option::Option::Some(Superstate::Blinking { counter })
            }
            State::Off {} => ::core::option::Option::None,
        }
    }
}
impl<'sub> ::sotu::awaitable::Superstate<Blinker> for Superstate<'sub> {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Blinker,
        event: &Event,
        _context: &mut (),
    ) -> impl ::core::future::Future<
        Output = ::sotu::Outcome<State, ::core::convert::Infallible>,
    > {
        async move {
            match self {
                Superstate::Outer { counter } => {
                    Blinker::outer(shared_storage, counter, event).await
                }
                Superstate::Blinking { counter } => {
                    Blinker::blinking(counter, event).await
                }
            }
        }
    }
    fn superstate(self) -> ::core::option::Option<Self> {
        match self {
            Superstate::Outer { .. } => ::core::option::Option::None,
            Superstate::Blinking { counter } => {
                ::core::option::Option::Some(Superstate::Outer { counter })
            }
        }
    }
    fn same_state(&self, other: &Superstate<'_>) -> bool {
        match (self, other) {
            (Superstate::Outer { .. }, Superstate::Outer { .. }) => true,
            (Superstate::Blinking { .. }, Superstate::Blinking { .. }) => true,
            _ => false,
        }
    }
}
impl ::sotu::private::Submachine for Blinker {
    type Event = Event;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Superstate<'sub> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::on(0)
    }
}
impl ::sotu::private::ExitlessSubmachine for Blinker {}
fn main() {
    let _machine = Blinker::default().state_machine();
}
