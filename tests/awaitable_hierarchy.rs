#![cfg(feature = "async")]

mod common;

use common::async_tree::async_state_for;
use common::tree::Leaf;
use sotu::awaitable::StateExt as _;

// MARK: Depth

#[test]
fn depth_of_every_leaf() {
    assert_eq!(async_state_for(Leaf::A1).depth(), 2);
    assert_eq!(async_state_for(Leaf::B1).depth(), 3);
    assert_eq!(async_state_for(Leaf::B2).depth(), 3);
    assert_eq!(async_state_for(Leaf::C).depth(), 1);
}
