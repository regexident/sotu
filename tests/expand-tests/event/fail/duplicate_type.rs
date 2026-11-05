use sotu::prelude::*;

mod common {
    pub struct Tick;
}

#[derive(Event)]
enum Event {
    A(common::Tick),
    B(common::Tick),
}

fn main() {}
