use sotu::prelude::*;
pub enum Light {
    Off {},
    On { level: u8 },
}
pub struct Lamp;
impl Lamp {
    fn off() -> Outcome<Light> {
        Transition(Light::On { level: 3 })
    }
    fn on(level: &mut u8) -> Outcome<Light> {
        *level += 1;
        Handled
    }
}
impl ::sotu::blocking::IntoStateMachine for Lamp {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = Light;
    type Superstate<'sub> = () where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> Light {
        Light::Off {}
    }
}
impl ::sotu::blocking::State<Lamp> for Light {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Lamp,
        _event: &(),
        _context: &mut (),
    ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
        match self {
            Light::Off {} => Lamp::off(),
            Light::On { level } => Lamp::on(level),
        }
    }
}
impl ::sotu::private::Submachine for Lamp {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = Light;
    type Superstate<'sub> = () where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> Light {
        Light::Off {}
    }
}
impl ::sotu::private::BlockingSubmachine for Lamp {}
fn main() {
    let mut machine = Lamp.state_machine();
    machine.step(&mut ());
}
