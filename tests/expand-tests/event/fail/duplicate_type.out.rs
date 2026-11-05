use sotu::prelude::*;
mod common {
    pub struct Tick;
}
enum Event {
    A(common::Tick),
    B(common::Tick),
}
fn main() {}
