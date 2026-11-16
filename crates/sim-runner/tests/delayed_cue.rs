use std::process::Command;

#[test]
fn cli_emits_only_the_machine_readable_instrument_result() {
    let result = Command::new(env!("CARGO_BIN_EXE_sim-runner"))
        .args(["delayed-cue", "--blank-delay", "7"])
        .output()
        .unwrap();
    assert!(result.status.success(), "{:?}", result);
    assert!(result.stderr.is_empty());
    let stdout = String::from_utf8(result.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.starts_with("{\"schema_version\":1,"));
    assert!(stdout.trim_end().ends_with('}'));
    assert!(stdout.contains("\"blank_delay_ticks\":7"));
    assert!(stdout.contains("\"instrument_validated\":true"));
    assert_eq!(stdout.matches("\"accuracy\":0.5").count(), 2);
    assert_eq!(stdout.matches("\"accuracy\":1").count(), 1);
}

#[test]
fn cli_rejects_excessive_delays_and_negative_values_without_results() {
    for delay in ["10001", "-1"] {
        let result = Command::new(env!("CARGO_BIN_EXE_sim-runner"))
            .args(["delayed-cue", "--blank-delay", delay])
            .output()
            .unwrap();
        assert!(!result.status.success());
        assert!(result.stdout.is_empty());
    }
}
