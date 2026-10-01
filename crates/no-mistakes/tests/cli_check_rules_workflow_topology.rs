use std::path::PathBuf;
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(name: &str) -> PathBuf {
    rule_fixture("concurrency-intent", name)
}

fn collision_fixture(name: &str) -> PathBuf {
    rule_fixture("group-collisions", name)
}

fn rule_fixture(suite: &str, name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/workflow-topology-policy/fixture")
            .join(suite)
            .join(name),
    )
}

fn check_json(name: &str) -> std::process::Output {
    check_root(&fixture(name))
}

fn check_collision_json(name: &str) -> std::process::Output {
    check_root(&collision_fixture(name))
}

fn check_root(root: &std::path::Path) -> std::process::Output {
    Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap()
}

#[test]
fn concurrency_intent_fail_json_is_stable_and_names_each_mismatch() {
    let first = check_json("fail");
    let second = check_json("fail");
    assert_eq!(first.stdout, second.stdout);
    assert!(
        !first.status.success(),
        "expected the fail fixture to exit 1"
    );
    let body = String::from_utf8(first.stdout).unwrap();
    for message in [
        "concurrency cancellation mismatch: .github/workflows/ci.yml: expected conditional, got cancel-running",
        "concurrency intent missing: .github/workflows/lint.yml",
        "concurrency pending mismatch: .github/workflows/release.yml#publish: expected fifo, got coalesce-latest",
        "concurrency scope mismatch: .github/workflows/deploy.yml: expected , got sha",
    ] {
        assert!(body.contains(message), "{message} missing from {body}");
    }
}

#[test]
fn concurrency_intent_pass_json_omits_concurrency_findings() {
    let output = check_json("pass");
    let body = String::from_utf8(output.stdout).unwrap();
    assert!(
        !body.contains("concurrency intent")
            && !body.contains("concurrency pending")
            && !body.contains("concurrency cancellation")
            && !body.contains("concurrency scope")
            && !body.contains("conditional cancel-in-progress"),
        "{body}"
    );
}

#[test]
fn group_collision_fail_json_is_stable_and_names_each_shared_group() {
    let first = check_collision_json("fail");
    let second = check_collision_json("fail");
    assert_eq!(first.stdout, second.stdout);
    assert!(
        !first.status.success(),
        "expected the fail fixture to exit 1"
    );
    let body = String::from_utf8(first.stdout).unwrap();
    for message in [
        "concurrency group collision: ${{ github.ref }}: .github/workflows/a.yml, .github/workflows/b.yml",
        "concurrency group collision: ${{ github.workflow }}-${{ github.ref }}: .github/workflows/ci.yml, .github/workflows/ci.yml#build",
        "concurrency group collision: shared: .github/workflows/one.yml, .github/workflows/two.yml",
    ] {
        assert!(body.contains(message), "{message} missing from {body}");
    }
}

#[test]
fn group_collision_pass_json_omits_collision_findings() {
    let output = check_collision_json("pass");
    let body = String::from_utf8(output.stdout).unwrap();
    assert!(!body.contains("concurrency group collision"), "{body}");
}
