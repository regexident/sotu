#![cfg(feature = "async")]

mod common;

use common::async_tree::{AsyncTree, AsyncTreeState};
use common::tree::{Node, Record};
use futures::executor::block_on;
use sotu::awaitable::IntoStateMachineExt;

// MARK: Initialization

#[test]
fn init_enters_initial_state_outermost_first() {
    block_on(async {
        let uninitialized = AsyncTree::default().uninitialized_state_machine();

        assert_eq!(uninitialized.trace, []);
        assert_eq!(uninitialized.state(), &AsyncTreeState::A1);

        let machine = uninitialized.init(&mut ()).await;

        assert_eq!(
            machine.trace,
            [Record::Entry(Node::A), Record::Entry(Node::A1)],
        );
        assert_eq!(machine.state(), &AsyncTreeState::A1);
    });
}
