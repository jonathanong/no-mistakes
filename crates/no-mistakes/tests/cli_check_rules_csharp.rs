use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(scenario: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/csharp-max-lines-per-file/fixture")
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
fn csharp_max_lines_per_file_passes_under_limit() {
    let root = fixture("pass");
    let out = check(
        &root,
        "rules:\n  - rule: csharp-max-lines-per-file\n    scope: repository\n    options:\n      srcMax: 20\n",
    );
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn csharp_max_lines_per_file_fails_over_limit() {
    let root = fixture("fail");
    let out = check(
        &root,
        "rules:\n  - rule: csharp-max-lines-per-file\n    scope: repository\n    options:\n      srcMax: 3\n",
    );
    assert!(!out.status.success(), "expected exit 1");
    assert!(stdout(&out).contains("physical lines"), "{}", stdout(&out));
    assert!(!stdout(&out).contains("code lines"), "{}", stdout(&out));
    assert!(stdout(&out).contains("TooLong.cs"), "{}", stdout(&out));
}

#[test]
fn csharp_max_lines_per_file_disabled_skips() {
    let root = fixture("fail");
    let out = check(
        &root,
        "rules:\n  - rule: csharp-max-lines-per-file\n    enabled: false\n    scope: repository\n    options:\n      srcMax: 3\n",
    );
    assert!(
        out.status.success(),
        "disabled rule must not fail: {}",
        stdout(&out)
    );
}

#[test]
fn csharp_max_lines_per_file_json_has_rule_id() {
    let root = fixture("fail");
    let config = tempfile::Builder::new().suffix(".yml").tempfile().unwrap();
    std::fs::write(
        config.path(),
        "rules:\n  - rule: csharp-max-lines-per-file\n    scope: repository\n    options:\n      srcMax: 3\n",
    )
    .unwrap();
    let out = Command::new(bin())
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(config.path())
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert!(
        stdout(&out).contains("csharp-max-lines-per-file"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn csharp_max_lines_per_file_skips_generated() {
    let root = fixture("generated");
    let out = check(
        &root,
        "rules:\n  - rule: csharp-max-lines-per-file\n    scope: repository\n    options:\n      srcMax: 3\n",
    );
    assert!(
        out.status.success(),
        "generated files should be excluded: {}",
        stdout(&out)
    );
}

#[test]
fn csharp_max_lines_per_file_uses_test_max() {
    let root = fixture("test");
    let out = check(
        &root,
        "rules:\n  - rule: csharp-max-lines-per-file\n    scope: repository\n    options:\n      srcMax: 3\n      testMax: 20\n",
    );
    assert!(
        out.status.success(),
        "test files should use testMax: {}",
        stdout(&out)
    );
}
