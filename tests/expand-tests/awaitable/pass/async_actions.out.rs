use sotu::prelude::*;
pub enum Event {
    Flip,
}
pub struct Lamp {
    pub flips: u32,
}
#[automatically_derived]
impl ::core::default::Default for Lamp {
    #[inline]
    fn default() -> Lamp {
        Lamp {
            flips: ::core::default::Default::default(),
        }
    }
}
impl Lamp {
    fn powered() -> Outcome<State> {
        Super
    }
    fn on(lit: &mut bool, event: &Event) -> Outcome<State> {
        match event {
            Event::Flip if *lit => Transition(State::off()),
            Event::Flip => Handled,
        }
    }
    fn off(event: &Event) -> Outcome<State> {
        match event {
            Event::Flip => Transition(State::on(false)),
        }
    }
    async fn power_up(&mut self) {
        self.flips += 1;
    }
    fn power_down(&mut self) {
        self.flips += 1;
    }
    async fn light(&mut self, lit: &mut bool) {
        *lit = true;
        self.flips += 1;
    }
    fn dim(lit: &mut bool) {
        *lit = false;
    }
}
pub enum State {
    On { lit: bool },
    Off {},
}
impl State {
    pub fn on(lit: bool) -> Self {
        Self::On { lit }
    }
    pub fn off() -> Self {
        Self::Off {}
    }
}
pub enum Superstate {
    Powered {},
}
impl ::sotu::awaitable::IntoStateMachine for Lamp {
    type Event = Event;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Superstate where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::off()
    }
}
impl ::sotu::awaitable::State<Lamp> for State {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Lamp,
        event: &Event,
        _context: &mut (),
    ) -> impl ::core::future::Future<
        Output = ::sotu::Outcome<Self, ::core::convert::Infallible>,
    > {
        async move {
            match self {
                State::On { lit } => Lamp::on(lit, event),
                State::Off {} => Lamp::off(event),
            }
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Lamp,
        _context: &mut (),
    ) -> impl ::core::future::Future<Output = ()> {
        async move {
            match self {
                State::On { lit } => Lamp::light(shared_storage, lit).await,
                State::Off {} => {}
            }
        }
    }
    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Lamp,
        _context: &mut (),
    ) -> impl ::core::future::Future<Output = ()> {
        async move {
            match self {
                State::On { lit } => Lamp::dim(lit),
                State::Off {} => {}
            }
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate> {
        match self {
            State::On { .. } => ::core::option::Option::Some(Superstate::Powered {}),
            State::Off {} => ::core::option::Option::None,
        }
    }
}
impl ::sotu::awaitable::Superstate<Lamp> for Superstate {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Lamp,
        _event: &Event,
        _context: &mut (),
    ) -> impl ::core::future::Future<
        Output = ::sotu::Outcome<State, ::core::convert::Infallible>,
    > {
        async move {
            match self {
                Superstate::Powered {} => Lamp::powered(),
            }
        }
    }
    fn call_entry_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Lamp,
        _context: &mut (),
    ) -> impl ::core::future::Future<Output = ()> {
        async move {
            match self {
                Superstate::Powered {} => Lamp::power_up(shared_storage).await,
            }
        }
    }
    fn call_exit_action<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Lamp,
        _context: &mut (),
    ) -> impl ::core::future::Future<Output = ()> {
        async move {
            match self {
                Superstate::Powered {} => Lamp::power_down(shared_storage),
            }
        }
    }
    fn superstate(self) -> ::core::option::Option<Self> {
        match self {
            Superstate::Powered {} => ::core::option::Option::None,
        }
    }
    fn same_state(&self, other: &Superstate) -> bool {
        match (self, other) {
            (Superstate::Powered { .. }, Superstate::Powered { .. }) => true,
        }
    }
}
impl ::sotu::private::Submachine for Lamp {
    type Event = Event;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State;
    type Superstate<'sub> = Superstate where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State {
        State::off()
    }
}
impl ::sotu::private::ExitlessSubmachine for Lamp {}
fn main() {
    let _machine = Lamp::default().state_machine();
}
