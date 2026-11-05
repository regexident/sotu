use sotu::prelude::*;

struct Tick;

type Alias = Tick;

#[derive(Event)]
enum Event {
    A(Tick),
    B(Alias),
}

fn main() {}
