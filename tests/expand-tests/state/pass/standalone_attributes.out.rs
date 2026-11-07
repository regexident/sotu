use sotu::prelude::*;
fn bare_state() -> u32 {
    1
}
fn state_with_arguments() -> u32 {
    2
}
fn bare_superstate() -> u32 {
    3
}
fn superstate_with_arguments() -> u32 {
    4
}
fn bare_submachine() -> u32 {
    5
}
fn submachine_with_arguments() -> u32 {
    6
}
fn bare_action() -> u32 {
    7
}
fn action_with_arguments() -> u32 {
    8
}
fn bare_exit_handler() -> u32 {
    9
}
fn exit_handler_with_arguments() -> u32 {
    10
}
fn main() {
    let total = bare_state() + state_with_arguments() + bare_superstate()
        + superstate_with_arguments() + bare_submachine() + submachine_with_arguments()
        + bare_action() + action_with_arguments() + bare_exit_handler()
        + exit_handler_with_arguments();
    match (&total, &55) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
}
