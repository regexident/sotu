use std::process::Command;

/// Run the example `name` with the given Cargo `features` and return its
/// standard output.
///
/// The example is started through Cargo, so the test builds exactly what a user
/// gets from `cargo run --example <name>`.
fn run_example(name: &str, features: &[&str]) -> String {
    let mut command = Command::new(env!("CARGO"));

    command
        .arg("run")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .arg("--example")
        .arg(name);

    if !features.is_empty() {
        command.arg("--features").arg(features.join(","));
    }

    let output = command.output().expect("cargo should start");

    // The caller compares the returned text. A failed run is reported here, so
    // the comparison failure comes with the example's own error output.
    if !output.status.success() {
        eprintln!(
            "example `{name}` exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr),
        );
    }

    String::from_utf8(output.stdout).expect("example output should be UTF-8")
}

#[test]
fn blinky_prints_trace() {
    assert_eq!(
        run_example("blinky", &[]),
        "enter blinking\n\
         enter led_on\n\
         exit led_on\n\
         enter led_off\n\
         exit led_off\n\
         enter led_on\n\
         exit led_on\n\
         exit blinking\n\
         enter not_blinking\n\
         exit not_blinking\n\
         enter blinking\n\
         enter led_on\n"
    );
}

#[test]
fn calculator_prints_results() {
    assert_eq!(
        run_example("calculator", &[]),
        "result: 42\n\
         result: 15\n"
    );
}

#[test]
fn composition_prints_states() {
    assert_eq!(
        run_example("composition", &[]),
        "start: Loading(Opened)\n\
         openings: 1\n\
         close: Washing\n\
         door present: false\n\
         tick: Idle\n\
         cycles: 1\n"
    );
}
