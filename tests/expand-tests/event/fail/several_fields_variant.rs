use sotu::prelude::*;

struct Tick;
struct Tock;

#[derive(Event)]
enum Event {
    Tick(Tick, Tock),
}

fn main() {}
