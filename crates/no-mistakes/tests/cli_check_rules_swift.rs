use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(scenario: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/swift-no-raw-print/fixture")
            .join(scenario),
    )
}

fn check(root: &PathBuf, yaml: &str) -> Output {
    let config = tempfile::Builder::new().suffix(".yml").tempfile().unwrap();
    std::fs::write(config.path(), yaml).unwrap();
    Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(config.path())
        .output()
        .unwrap()
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

#[test]
fn swift_no_raw_print_cli_fails() {
    let root = fixture("fail");
    let out = check(
        &root,
        "rules:\n  - rule: swift-no-raw-print\n    scope: repository\n",
    );
    assert!(!out.status.success(), "expected exit 1");
    assert!(
        stdout(&out).contains("swift-no-raw-print"),
        "{}",
        stdout(&out)
    );
    assert!(stdout(&out).contains("FeedView.swift"), "{}", stdout(&out));
}

#[test]
fn swift_no_raw_print_cli_passes() {
    let root = fixture("pass");
    let out = check(
        &root,
        "rules:\n  - rule: swift-no-raw-print\n    scope: repository\n",
    );
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

fn viewmodel_fixture(scenario: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/swift-viewmodel-main-actor/fixture")
            .join(scenario),
    )
}

#[test]
fn swift_viewmodel_main_actor_cli_fails() {
    let root = viewmodel_fixture("fail");
    let out = check(
        &root,
        "rules:\n  - rule: swift-viewmodel-main-actor\n    scope: repository\n",
    );
    assert!(!out.status.success(), "expected exit 1");
    assert!(
        stdout(&out).contains("swift-viewmodel-main-actor"),
        "{}",
        stdout(&out)
    );
    assert!(
        stdout(&out).contains("BrokenViewModel.swift"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn swift_viewmodel_main_actor_cli_passes() {
    let root = viewmodel_fixture("pass");
    let out = check(
        &root,
        "rules:\n  - rule: swift-viewmodel-main-actor\n    scope: repository\n",
    );
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}
