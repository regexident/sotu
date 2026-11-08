mod event {
    #[test]
    pub fn pass() {
        tryexpand::expand(["tests/expand-tests/event/pass/**/*.rs"]).and_check();
    }

    #[test]
    pub fn fail() {
        tryexpand::expand(["tests/expand-tests/event/fail/**/*.rs"])
            .and_check()
            .expect_fail();
    }
}

mod state_machine {
    #[test]
    pub fn pass() {
        tryexpand::expand(["tests/expand-tests/state_machine/pass/**/*.rs"]).and_check();
    }
}

mod state {
    #[test]
    pub fn pass() {
        tryexpand::expand(["tests/expand-tests/state/pass/**/*.rs"]).and_check();
    }
}

mod superstate {
    #[test]
    pub fn pass() {
        tryexpand::expand(["tests/expand-tests/superstate/pass/**/*.rs"]).and_check();
    }
}

mod action {
    #[test]
    pub fn pass() {
        tryexpand::expand(["tests/expand-tests/action/pass/**/*.rs"]).and_check();
    }
}
