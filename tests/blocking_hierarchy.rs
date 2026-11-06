mod common;

use common::tree::{Leaf, state_for};
use sotu::blocking::StateExt;

// MARK: Depth

#[test]
fn depth_of_every_leaf() {
    assert_eq!(state_for(Leaf::A1).depth(), 2);
    assert_eq!(state_for(Leaf::B1).depth(), 3);
    assert_eq!(state_for(Leaf::B2).depth(), 3);
    assert_eq!(state_for(Leaf::C).depth(), 1);
}
