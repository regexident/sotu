use sotu::event::SubEvent;
use sotu::prelude::*;

mod inner {
    pub struct Tick;

    pub mod deeper {
        pub struct Tock;
    }
}

#[derive(Event)]
enum Event {
    Tick(inner::Tick),
    Tock(inner::deeper::Tock),
}

fn requires<C: SubEvent<P>, P>() {}

fn main() {
    requires::<Event, Event>();

    let _tick: Event = inner::Tick.into();
    let _tock: Event = inner::deeper::Tock.into();
}
