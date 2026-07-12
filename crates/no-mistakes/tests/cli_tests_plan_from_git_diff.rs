//! Covers `tests plan --from-git-diff`: single-argument sugar over
//! `--base`/`--head` that desugars to the same `git diff --name-status
//! <base>...<head>` lookup (see `tests/changed_files.rs::parse_git_diff_refspec`).

use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture_dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/tests-plan-from-git-diff")
        .join(name)
}

// Shares the `workspace-package-bump` fixture with cli_tests_plan_lockfile2.rs
// (same repo-relative fixture files, no duplication) to prove --from-git-diff
// traces lockfile package changes exactly like --base/--head.
fn lockfile_fixture_dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/tests-plan-lockfile")
        .join(name)
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let ty = entry.file_type().unwrap();
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()));
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name())).unwrap();
        }
    }
}

fn setup_git_repo(root: &std::path::Path) {
    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(root)
        .output()
        .unwrap();
    for (k, v) in [("user.email", "test@test.com"), ("user.name", "Test")] {
        Command::new("git")
            .args(["config", k, v])
            .current_dir(root)
            .output()
            .unwrap();
    }
    git_commit_all(root, "initial");
}

fn git_commit_all(root: &std::path::Path, message: &str) {
    Command::new("git")
        .args(["add", "-A"])
        .current_dir(root)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(root)
        .output()
        .unwrap();
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf8")
}

// Sets up two commits (an initial fixture commit, then a commit that changes
// helper.mts) and returns the tempdir so the caller can run `tests plan`
// against it with a refspec spanning the two commits. Mirrors the
// initial/-plus-sibling-"after"-file layout used by
// tests-plan-lockfile/workspace-package-bump: the "after" content lives
// outside `basic/initial/` so copy_dir_all doesn't pull it into the
// first commit.
fn two_commit_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    copy_dir_all(&fixture_dir("basic/initial"), root);
    setup_git_repo(root);

    std::fs::copy(
        fixture_dir("basic/after-helper.mts"),
        root.join("helper.mts"),
    )
    .unwrap();
    git_commit_all(root, "change helper");

    dir
}

#[test]
fn from_git_diff_three_dot_selects_impacted_test() {
    let dir = two_commit_repo();
    let root = dir.path();

    let output = Command::new(bin())
        .args([
            "tests",
            "plan",
            "vitest",
            "--root",
            root.to_str().unwrap(),
            "--from-git-diff",
            "HEAD~1...HEAD",
            "--json",
        ])
        .output()
        .expect("no-mistakes should run");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let selected = plan["selected_tests"].as_array().unwrap();
    assert!(
        selected.iter().any(|t| t["test_file"] == "helper.test.mts"),
        "should find helper.test.mts via --from-git-diff HEAD~1...HEAD: {selected:?}"
    );
}

// The whole point of --from-git-diff is that it desugars to the same
// git diff --name-status lookup as --base/--head, so the two invocations
// must select the exact same tests.
#[test]
fn from_git_diff_matches_equivalent_base_head() {
    let dir = two_commit_repo();
    let root = dir.path();

    let from_git_diff_output = Command::new(bin())
        .args([
            "tests",
            "plan",
            "vitest",
            "--root",
            root.to_str().unwrap(),
            "--from-git-diff",
            "HEAD~1...HEAD",
            "--json",
        ])
        .output()
        .expect("no-mistakes should run");
    let base_head_output = Command::new(bin())
        .args([
            "tests",
            "plan",
            "vitest",
            "--root",
            root.to_str().unwrap(),
            "--base",
            "HEAD~1",
            "--head",
            "HEAD",
            "--json",
        ])
        .output()
        .expect("no-mistakes should run");

    assert!(from_git_diff_output.status.success());
    assert!(base_head_output.status.success());

    let from_git_diff_plan: serde_json::Value =
        serde_json::from_str(&stdout(&from_git_diff_output)).unwrap();
    let base_head_plan: serde_json::Value =
        serde_json::from_str(&stdout(&base_head_output)).unwrap();
    assert_eq!(
        from_git_diff_plan["selected_tests"], base_head_plan["selected_tests"],
        "--from-git-diff HEAD~1...HEAD must select the same tests as --base HEAD~1 --head HEAD"
    );
}

#[test]
fn from_git_diff_bare_base_defaults_head_to_head() {
    let dir = two_commit_repo();
    let root = dir.path();

    let output = Command::new(bin())
        .args([
            "tests",
            "plan",
            "vitest",
            "--root",
            root.to_str().unwrap(),
            "--from-git-diff",
            "HEAD~1",
            "--json",
        ])
        .output()
        .expect("no-mistakes should run");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let selected = plan["selected_tests"].as_array().unwrap();
    assert!(
        selected.iter().any(|t| t["test_file"] == "helper.test.mts"),
        "bare base should default head to HEAD: {selected:?}"
    );
}

#[test]
fn from_git_diff_rejects_two_dot_refspec() {
    let root = fixture_dir("basic/initial");

    let output = Command::new(bin())
        .args([
            "tests",
            "plan",
            "--root",
            root.to_str().unwrap(),
            "--from-git-diff",
            "HEAD~1..HEAD",
            "--json",
        ])
        .output()
        .expect("no-mistakes should run");

    assert!(!output.status.success(), "two-dot refspec must be rejected");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("two-dot"),
        "expected two-dot guidance in stderr: {stderr}"
    );
}

#[test]
fn from_git_diff_conflicts_with_base() {
    let root = fixture_dir("basic/initial");

    let output = Command::new(bin())
        .args([
            "tests",
            "plan",
            "--root",
            root.to_str().unwrap(),
            "--from-git-diff",
            "HEAD~1...HEAD",
            "--base",
            "HEAD~1",
            "--json",
        ])
        .output()
        .expect("no-mistakes should run");

    assert!(
        !output.status.success(),
        "--from-git-diff and --base must conflict on the CLI"
    );
}

// Regression for a review finding on #508: analyze_lockfile_changes reads
// args.base/args.head directly, so --from-git-diff must resolve into those
// fields (in generate_plan, before collect_changed_files/lockfile analysis
// run) rather than only being consulted locally inside the changed-file
// git-diff lookup. Otherwise a diff that bumps a workspace package would
// silently lose lockfile-package tracing and fall back with
// lockfile-no-baseline under --from-git-diff even though the equivalent
// --base/--head invocation traces it correctly.
#[test]
fn from_git_diff_traces_lockfile_package_bump_like_base_head() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    copy_dir_all(
        &lockfile_fixture_dir("workspace-package-bump/initial"),
        root,
    );
    setup_git_repo(root);

    std::fs::copy(
        lockfile_fixture_dir("workspace-package-bump/after-pnpm-lock.yaml"),
        root.join("pnpm-lock.yaml"),
    )
    .unwrap();
    git_commit_all(root, "bump workspace package");

    let output = Command::new(bin())
        .args([
            "tests",
            "plan",
            "--root",
            root.to_str().unwrap(),
            "--changed-file",
            "pnpm-lock.yaml",
            "--from-git-diff",
            "HEAD~1...HEAD",
            "--json",
        ])
        .output()
        .expect("no-mistakes should run");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let warnings = plan["warnings"].as_array().unwrap();
    assert!(
        !warnings
            .iter()
            .any(|w| w["type"] == "lockfile-no-baseline"),
        "--from-git-diff must resolve base/head for lockfile analysis, not just changed-file lookup: {warnings:?}"
    );
    assert!(
        !plan["fallback_triggered"].as_bool().unwrap(),
        "workspace package bump should be traceable via --from-git-diff, not fall back: {plan:?}"
    );
    let selected = plan["selected_tests"].as_array().unwrap();
    assert!(
        selected
            .iter()
            .any(|t| t["test_file"].as_str().unwrap().contains("utils.test")),
        "should trace workspace package lib to utils.test.ts via --from-git-diff: {selected:?}"
    );
}

// Regression for a further review finding on #508: the bare-base form
// (`--from-git-diff <base>`, no explicit head) must resolve head to the
// literal ref "HEAD" for *every* consumer of args.head, not just
// collect_changed_files (which already defaults a None head to "HEAD" via
// unwrap_or). analyze_lockfile_changes treats a None head differently — it
// reads the lockfile from the *working tree* instead of `git show HEAD`. To
// tell those two interpretations apart, this test deliberately makes the
// working tree disagree with HEAD after the "bump" commit (an uncommitted
// revert back to the "before" lockfile) and asserts the plan still traces
// the package bump from HEAD's committed content, not the stale disk state.
#[test]
fn from_git_diff_bare_base_traces_lockfile_from_head_not_working_tree() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    copy_dir_all(
        &lockfile_fixture_dir("workspace-package-bump/initial"),
        root,
    );
    setup_git_repo(root);

    std::fs::copy(
        lockfile_fixture_dir("workspace-package-bump/after-pnpm-lock.yaml"),
        root.join("pnpm-lock.yaml"),
    )
    .unwrap();
    git_commit_all(root, "bump workspace package");

    // Uncommitted: revert the working tree back to the "before" lockfile, so
    // disk and HEAD disagree. If head resolution stayed None (working-tree
    // read), this would make the lockfile diff look empty and the consumer
    // test would not be selected.
    std::fs::copy(
        lockfile_fixture_dir("workspace-package-bump/initial/pnpm-lock.yaml"),
        root.join("pnpm-lock.yaml"),
    )
    .unwrap();

    let output = Command::new(bin())
        .args([
            "tests",
            "plan",
            "--root",
            root.to_str().unwrap(),
            "--changed-file",
            "pnpm-lock.yaml",
            "--from-git-diff",
            "HEAD~1",
            "--json",
        ])
        .output()
        .expect("no-mistakes should run");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let warnings = plan["warnings"].as_array().unwrap();
    assert!(
        !warnings
            .iter()
            .any(|w| w["type"] == "lockfile-no-baseline"),
        "bare-base --from-git-diff must resolve head to HEAD for lockfile analysis too: {warnings:?}"
    );
    let selected = plan["selected_tests"].as_array().unwrap();
    assert!(
        selected
            .iter()
            .any(|t| t["test_file"].as_str().unwrap().contains("utils.test")),
        "bare-base --from-git-diff should trace the bump from HEAD's lockfile content, \
         not the working tree: {selected:?}"
    );
}
