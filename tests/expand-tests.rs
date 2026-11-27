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

    #[test]
    pub fn fail() {
        tryexpand::expand(["tests/expand-tests/state_machine/fail/**/*.rs"])
            .and_check()
            .expect_fail();
    }
}

mod state {
    #[test]
    pub fn pass() {
        tryexpand::expand(["tests/expand-tests/state/pass/**/*.rs"]).and_check();
    }

    #[test]
    pub fn fail() {
        tryexpand::expand(["tests/expand-tests/state/fail/**/*.rs"])
            .and_check()
            .expect_fail();
    }
}

mod superstate {
    #[test]
    pub fn pass() {
        tryexpand::expand(["tests/expand-tests/superstate/pass/**/*.rs"]).and_check();
    }

    #[test]
    pub fn fail() {
        tryexpand::expand(["tests/expand-tests/superstate/fail/**/*.rs"])
            .and_check()
            .expect_fail();
    }
}

mod submachine {
    #[test]
    pub fn pass() {
        tryexpand::expand(["tests/expand-tests/submachine/pass/**/*.rs"]).and_check();
    }

    #[test]
    pub fn fail() {
        // The number of errors that cargo reports for the submachine bounds
        // includes duplicates that rustc suppresses, and how many of those
        // it counts differs between host platforms.
        tryexpand::expand(["tests/expand-tests/submachine/fail/**/*.rs"])
            .and_check()
            .filter_stderr(r"due to \d+ previous errors", "due to <N> previous errors")
            .expect_fail();
    }
}

mod action {
    #[test]
    pub fn pass() {
        tryexpand::expand(["tests/expand-tests/action/pass/**/*.rs"]).and_check();
    }

    #[test]
    pub fn fail() {
        tryexpand::expand(["tests/expand-tests/action/fail/**/*.rs"])
            .and_check()
            .expect_fail();
    }
}

mod serde {
    #[test]
    pub fn pass() {
        // The expansion crate sees only the regular dependencies, which do not
        // enable the derive macros of `serde`. Their expansions name a private
        // module of `serde` that carries its release number.
        tryexpand::expand(["tests/expand-tests/serde/pass/**/*.rs"])
            .args(["--features", "serde,serde/derive"])
            .filter_stdout(r"__private\d+", "__private")
            .and_check();
    }
}

mod awaitable {
    #[test]
    pub fn pass() {
        tryexpand::expand(["tests/expand-tests/awaitable/pass/**/*.rs"])
            .args(["--features", "async"])
            .and_check();
    }

    #[test]
    pub fn fail() {
        // The number of errors that cargo reports for the awaitable bounds
        // includes duplicates that rustc suppresses, and how many of those
        // it counts differs between host platforms.
        tryexpand::expand(["tests/expand-tests/awaitable/fail/**/*.rs"])
            .args(["--features", "async"])
            .and_check()
            .filter_stderr(r"due to \d+ previous errors", "due to <N> previous errors")
            .expect_fail();
    }
}
