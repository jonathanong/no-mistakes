use super::*;

#[test]
fn discovers_deleted_locks_and_skips_undecodable_revisions() {
    let repo = empty_repo();
    let root = repo.path();
    let lock = root.join("Cargo.lock");
    std::fs::copy(ecosystem_lock("cargo/base.lock"), &lock).unwrap();
    git_commit(root, &["Cargo.lock"], "valid");
    std::fs::copy(ecosystem_lock("cargo/unreadable.lock"), &lock).unwrap();
    git_commit(root, &["Cargo.lock"], "unreadable");

    let output = diff_revisions(root, "HEAD~1");
    assert!(output.status.success());
    let warning = String::from_utf8_lossy(&output.stderr);
    assert!(
        warning.contains("`Cargo.lock` at ref `HEAD` is not valid UTF-8"),
        "{warning}"
    );
    let report: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(report, serde_json::json!([]));

    std::fs::remove_file(lock).unwrap();
    git_commit(root, &["Cargo.lock"], "deleted");
    let output = diff_revisions(root, "HEAD~2");
    assert!(output.status.success());
    let report: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(report[0]["manager"], "cargo");
    assert!(!report[0]["removed"].as_array().unwrap().is_empty());
}

fn diff_revisions(root: &Path, base: &str) -> Output {
    run(&[
        "lockfile",
        "diff",
        "--root",
        root.to_str().unwrap(),
        "--base",
        base,
        "--head",
        "HEAD",
        "--format",
        "json",
    ])
}
