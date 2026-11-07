use sotu::prelude::*;

#[state]
fn bare_state() -> u32 {
    1
}

#[state(superstate = "parent", entry_action = "enter")]
fn state_with_arguments() -> u32 {
    2
}

#[superstate]
fn bare_superstate() -> u32 {
    3
}

#[superstate(superstate = "parent", exit_action = "leave")]
fn superstate_with_arguments() -> u32 {
    4
}

#[submachine]
fn bare_submachine() -> u32 {
    5
}

#[submachine(superstate = "parent")]
fn submachine_with_arguments() -> u32 {
    6
}

#[action]
fn bare_action() -> u32 {
    7
}

#[action(name = "enter")]
fn action_with_arguments() -> u32 {
    8
}

#[exit_handler]
fn bare_exit_handler() -> u32 {
    9
}

#[exit_handler(submachine = "child")]
fn exit_handler_with_arguments() -> u32 {
    10
}

fn main() {
    let total = bare_state()
        + state_with_arguments()
        + bare_superstate()
        + superstate_with_arguments()
        + bare_submachine()
        + submachine_with_arguments()
        + bare_action()
        + action_with_arguments()
        + bare_exit_handler()
        + exit_handler_with_arguments();

    assert_eq!(total, 55);
}
