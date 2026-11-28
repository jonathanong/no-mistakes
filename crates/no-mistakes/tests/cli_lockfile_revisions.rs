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

#[test]
fn truncated_lock_is_malformed_at_a_git_revision() {
    let repo = empty_repo();
    let root = repo.path();
    let lock = root.join("composer.lock");
    std::fs::copy(ecosystem_lock("composer/base.lock"), &lock).unwrap();
    git_commit(root, &["composer.lock"], "valid");
    std::fs::copy(ecosystem_lock("composer/truncated.lock"), &lock).unwrap();
    git_commit(root, &["composer.lock"], "truncated");

    let output = diff_revisions(root, "HEAD~1");
    assert!(output.status.success());
    let warning = String::from_utf8_lossy(&output.stderr);
    assert!(
        warning.contains("malformed composer.lock content"),
        "{warning}"
    );
    let report: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(report, serde_json::json!([]));
}

#[cfg(unix)]
#[test]
fn unix_root_components_keep_literal_backslashes() {
    let dir = committed_repo("lockfile-diff-pnpm-subdir");
    let root = dir.path();
    let original = root.join("packages/api");
    let renamed = root.join(r"packages/service\api");
    std::fs::rename(&original, &renamed).unwrap();
    git_commit(root, &["-A"], "rename");
    copy_after("lockfile-diff-pnpm-subdir", &renamed.join("pnpm-lock.yaml"));
    git_commit(root, &["-A"], "update");

    let output = diff_revisions(&renamed, "HEAD~1");
    assert!(output.status.success());
    let report: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(
        report[0]["lockfile"],
        r"packages/service\api/pnpm-lock.yaml"
    );
    assert_eq!(report[0]["changed"], serde_json::json!(["lodash"]));
}

#[test]
fn a_git_tree_at_a_lockfile_path_is_reported_as_unreadable() {
    let fixture = ecosystem_lock("revision-directory");
    let dir = crate::saved_fixture::materialize_path(&fixture);
    let root = dir.path();
    git_init(root);
    git_commit(root, &["-A"], "directory");

    let output = diff_revisions(root, "HEAD");
    assert!(output.status.success());
    let warning = String::from_utf8_lossy(&output.stderr);
    assert!(
        warning.contains("`Cargo.lock` at ref `HEAD` could not be read as a file"),
        "{warning}"
    );
    let report: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(report, serde_json::json!([]));
}

#[test]
fn a_ref_pointing_to_a_blob_cannot_be_used_as_a_tree() {
    let dir = empty_repo();
    let root = dir.path();
    std::fs::copy(ecosystem_lock("cargo/base.lock"), root.join("Cargo.lock")).unwrap();
    git_commit(root, &["Cargo.lock"], "package data");

    let blob = Command::new("git")
        .args(["rev-parse", "HEAD:Cargo.lock"])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(blob.status.success());
    let blob = stdout(&blob);
    let discovery = diff_revisions(root, blob.trim());
    assert!(!discovery.status.success());
    assert!(String::from_utf8_lossy(&discovery.stderr).contains("could not list"));

    let output = run(&[
        "lockfile",
        "diff",
        "--root",
        root.to_str().unwrap(),
        "--base",
        blob.trim(),
        "--head",
        "HEAD",
        "--lockfile",
        "Cargo.lock",
    ]);
    let warning = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{warning}");
    assert!(warning.contains("could not list `Cargo.lock`"), "{warning}");
    let report: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(report, serde_json::json!([]));
}
