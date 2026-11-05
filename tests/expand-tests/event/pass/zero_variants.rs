use sotu::event::SubEvent;
use sotu::prelude::*;

#[derive(Event)]
enum Event {}

#[derive(Event)]
enum Left {
    Number(u8),
}

#[derive(Event)]
enum Right {
    Flag(bool),
}

fn requires<C: SubEvent<P>, P>() {}

fn main() {
    requires::<Event, Left>();
    requires::<Event, Right>();
    requires::<Event, Event>();

    let _number: Left = 1u8.into();
    let _flag: Right = true.into();
}
