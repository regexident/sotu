use sotu::event::SubEvent;
use sotu::prelude::*;

struct Start;
struct Stop;
struct Reset;

#[derive(Event)]
enum Parent {
    Start(Start),
    Stop(Stop),
    Reset(Reset),
}

#[derive(Event)]
enum Child {
    Start(Start),
    Stop(Stop),
}

fn requires<C: SubEvent<P>, P>() {}

fn main() {
    requires::<Child, Parent>();
    requires::<Parent, Parent>();

    let _start: Parent = Start.into();
    let _stop: Parent = Stop.into();
    let _reset: Parent = Reset.into();
}
