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
