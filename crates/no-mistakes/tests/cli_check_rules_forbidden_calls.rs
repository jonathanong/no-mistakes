use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture() -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/forbidden-calls/malformed-roots/fixture"),
    )
}

fn check(root: &Path, config: &Path) -> Output {
    Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(config)
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn forbidden_calls_valid_root_ignores_unrelated_parse_errors() {
    let root = fixture();
    let out = check(&root, &root.join(".no-mistakes.yml"));
    let body = format!("{}{}", stdout(&out), stderr(&out));
    assert!(body.contains("setTimeout"), "{body}");
    assert!(!body.contains("failed to parse"), "{body}");
    assert!(!body.contains("unrelated-broken"), "{body}");
}

#[test]
fn forbidden_calls_malformed_root_fails_closed() {
    let root = fixture();
    let config = tempfile::Builder::new().suffix(".yml").tempfile().unwrap();
    std::fs::write(
        config.path(),
        "rules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{ file: src/broken.mts }]\n      targets: [{ global: setTimeout }]\n",
    )
    .unwrap();
    let out = check(&root, config.path());
    let body = format!("{}{}", stdout(&out), stderr(&out));
    assert!(!out.status.success(), "expected failure: {body}");
    assert!(body.contains("failed to parse"), "{body}");
    assert!(body.contains("src/broken.mts"), "{body}");
}
