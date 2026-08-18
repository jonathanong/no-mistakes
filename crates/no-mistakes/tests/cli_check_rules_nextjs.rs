use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(category: &str, scenario: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules")
            .join(category)
            .join("fixture")
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

fn check_fixture_config(root: &PathBuf, name: &str) -> Output {
    let yaml = std::fs::read_to_string(root.join(name)).unwrap();
    check(root, &yaml)
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn nextjs_redirect_destinations_passes_for_existing_page() {
    let root = fixture("nextjs-redirect-destinations", "pass");
    let out = check_fixture_config(&root, ".no-mistakes.yml");
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn nextjs_redirect_destinations_fails_for_missing_page() {
    let root = fixture("nextjs-redirect-destinations", "fail-missing");
    let out = check_fixture_config(&root, ".no-mistakes.yml");
    let body = stdout(&out);

    assert!(!out.status.success(), "expected exit 1");
    assert!(body.contains("nextjs-redirect-destinations"), "{body}");
    assert!(body.contains("/gone"), "{body}");
}

#[test]
fn nextjs_redirect_destinations_flags_private_underscore_routes() {
    let root = fixture("nextjs-redirect-destinations", "skip-private");
    let out = check_fixture_config(&root, ".no-mistakes.yml");
    let body = stdout(&out);

    assert!(!out.status.success(), "expected exit 1");
    assert!(body.contains("nextjs-redirect-destinations"), "{body}");
    assert!(body.contains("/secret"), "{body}");
}

#[test]
fn nextjs_redirect_destinations_checks_rewrites_by_default() {
    let root = fixture("nextjs-redirect-destinations", "rewrite-fail");
    let out = check_fixture_config(&root, ".no-mistakes.yml");
    let body = stdout(&out);

    assert!(!out.status.success(), "expected exit 1");
    assert!(body.contains("nextjs-redirect-destinations"), "{body}");
    assert!(body.contains("rewrite destination"), "{body}");
}

#[test]
fn nextjs_redirect_destinations_flags_extractor_drift() {
    let root = fixture("nextjs-redirect-destinations", "fail-extractor");
    let out = check_fixture_config(&root, ".no-mistakes.yml");
    let body = stdout(&out);

    assert!(!out.status.success(), "expected exit 1");
    assert!(body.contains("nextjs-redirect-destinations"), "{body}");
    assert!(
        body.contains("could not locate the redirects() body"),
        "{body}"
    );
}
