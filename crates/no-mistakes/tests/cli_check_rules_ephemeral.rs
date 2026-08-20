use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(scenario: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/no-raw-ephemeral-port/fixture")
            .join(scenario),
    )
}

fn check_fixture_config(root: &PathBuf, name: &str) -> Output {
    Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(root.join(name))
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn no_raw_ephemeral_port_fails_for_python_bind_zero() {
    let root = fixture("fail-python");
    let findings = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    let body = format!("{findings:?}");
    assert!(!findings.is_empty(), "expected findings");
    assert!(body.contains("no-raw-ephemeral-port"), "{body}");
    assert!(body.contains("server.py"), "{body}");
}

#[test]
fn no_raw_ephemeral_port_fails_for_listen_zero() {
    let root = fixture("fail-listen");
    let findings = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    let body = format!("{findings:?}");
    assert!(!findings.is_empty(), "expected findings");
    assert!(body.contains("no-raw-ephemeral-port"), "{body}");
    assert!(body.contains("server.ts"), "{body}");
}

#[test]
fn no_raw_ephemeral_port_passes_for_non_zero_ports() {
    let root = fixture("pass");
    let findings = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn no_raw_ephemeral_port_passes_for_allowlisted_binder() {
    let root = fixture("allow");
    let findings = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn no_raw_ephemeral_port_cli_fails_for_listen_zero() {
    let root = fixture("fail-listen");
    let output = check_fixture_config(&root, ".no-mistakes.yml");
    let body = stdout(&output);
    assert!(!output.status.success(), "expected exit 1");
    assert!(body.contains("no-raw-ephemeral-port"), "{body}");
    assert!(
        body.contains("raw ephemeral port 0 bind/listen can occupy a deterministic runner slice"),
        "{body}"
    );
}

#[test]
fn no_raw_ephemeral_port_cli_passes_for_non_zero_ports() {
    let root = fixture("pass");
    let output = check_fixture_config(&root, ".no-mistakes.yml");
    assert!(output.status.success(), "{}", stdout(&output));
}
