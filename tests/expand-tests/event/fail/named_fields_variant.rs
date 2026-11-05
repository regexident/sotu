use sotu::prelude::*;

struct Tick;

#[derive(Event)]
enum Event {
    Tick { tick: Tick },
}

fn main() {}
