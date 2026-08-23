//! Covers streamed unified-diff hunks for `--base`/`--head` and
//! `--from-git-diff` (#583): rename/delete facts, Playwright selector-hint
//! selection, lockfile dependency impact, a >10 MB patch, and Git-input
//! diagnostics — all backed by the exact fixture shape the issue specifies.
//! `cli_tests_plan_from_git_diff.rs` covers refspec desugaring; the streaming
//! producer's own error-classification and byte-level behavior are unit
//! tested in `src/tests/git_diff.rs` and `src/invocation/child/stream.rs`.

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
        .args(["init", "-q", "-b", "main"])
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
        .args(["commit", "-q", "-m", message])
        .current_dir(root)
        .output()
        .unwrap();
}

fn git(args: &[&str], root: &std::path::Path) -> Output {
    Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf8")
}

fn plan_json(output: &Output) -> serde_json::Value {
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_str(&stdout(output)).unwrap()
}

fn selected_test_names(plan: &serde_json::Value) -> Vec<String> {
    plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["test_file"].as_str().unwrap().to_string())
        .collect()
}

/// Builds the two-commit repo matching the issue's fixture: renames
/// `helper.test.ts`, deletes `removed.ts`, changes a selector literal in
/// `selector.tsx`, and bumps the `lib` workspace package (and its lockfile
/// importer entry).
fn two_commit_hunks_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    copy_dir_all(&fixture_dir("hunks/initial"), root);
    setup_git_repo(root);

    git(
        &["mv", "src/helper.test.ts", "src/helper.renamed.test.ts"],
        root,
    );
    git(&["rm", "-q", "src/removed.ts"], root);
    std::fs::copy(
        fixture_dir("hunks/after-selector.tsx"),
        root.join("src/selector.tsx"),
    )
    .unwrap();
    std::fs::copy(
        fixture_dir("hunks/after-pnpm-lock.yaml"),
        root.join("pnpm-lock.yaml"),
    )
    .unwrap();
    git_commit_all(root, "head");
    dir
}

#[test]
fn base_head_matches_diff_stdin_of_the_equivalent_piped_diff() {
    let dir = two_commit_hunks_repo();
    let root = dir.path();

    let base_head = Command::new(bin())
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
        .unwrap();
    let base_head_plan = plan_json(&base_head);

    // Combined mode: piping the equivalent diff through `--diff-stdin`
    // *alongside* `--base`/`--head` (so lockfile analysis still has a ref
    // pair to read the old/new lockfile content from — pure `--diff-stdin`
    // with no base/head cannot determine that) must select the exact same
    // tests, proving the streamed hunks match the piped ones byte-for-byte.
    let diff_output = git(&["diff", "HEAD~1...HEAD"], root);
    assert!(diff_output.status.success());
    let mut diff_stdin_cmd = Command::new(bin());
    diff_stdin_cmd
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
            "--diff-stdin",
            "--json",
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut child = diff_stdin_cmd.spawn().unwrap();
    use std::io::Write;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&diff_output.stdout)
        .unwrap();
    let diff_stdin_output = child.wait_with_output().unwrap();
    let diff_stdin_plan = plan_json(&diff_stdin_output);

    assert_eq!(base_head_plan, diff_stdin_plan);

    // --from-git-diff must desugar to the identical plan too.
    let from_git_diff = Command::new(bin())
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
        .unwrap();
    assert_eq!(plan_json(&from_git_diff), base_head_plan);
}

// Deleted-file tracing (`trace_deleted_files`) only runs in the
// non-framework (union) planner, so this test omits the `vitest` framework
// argument — a framework-scoped plan (`tests plan vitest ...`) never emits
// `deleted-file` warnings, in either the git-streamed or inline-diff path.
#[test]
fn reports_rename_and_deletion_facts() {
    let dir = two_commit_hunks_repo();
    let root = dir.path();

    let output = Command::new(bin())
        .args([
            "tests",
            "plan",
            "--root",
            root.to_str().unwrap(),
            "--base",
            "HEAD~1",
            "--head",
            "HEAD",
            "--json",
        ])
        .output()
        .unwrap();
    let plan = plan_json(&output);
    let selected = selected_test_names(&plan);

    assert!(
        selected.contains(&"src/helper.renamed.test.ts".to_string()),
        "renamed test file should be selected: {selected:?}"
    );
    // `trace_deleted_files` emits a `deleted-file` warning for every deleted
    // path; it does not additionally select an importer whose only
    // reference was the now-deleted file (the same `--diff`-based behavior
    // as the existing `test_plan_diff_deleted_file_emits_warning` fixture)
    // — the deleted path can't be resolved into a graph edge once it no
    // longer exists on disk, so this matches inline-diff planning rather
    // than being a git-specific gap.
    let warnings = plan["warnings"].as_array().unwrap();
    assert!(
        warnings
            .iter()
            .any(|w| w["type"] == "deleted-file" && w["file"] == "src/removed.ts"),
        "deleted removed.ts should emit a deleted-file warning: {warnings:?}"
    );
}

#[test]
fn selects_playwright_spec_via_selector_hint_but_not_unrelated_spec() {
    let dir = two_commit_hunks_repo();
    let root = dir.path();

    let output = Command::new(bin())
        .args([
            "tests",
            "plan",
            "playwright",
            "--root",
            root.to_str().unwrap(),
            "--base",
            "HEAD~1",
            "--head",
            "HEAD",
            "--json",
        ])
        .output()
        .unwrap();
    let plan = plan_json(&output);
    let selected = selected_test_names(&plan);

    assert!(
        selected.iter().any(|t| t.contains("selector.spec.ts")),
        "the changed selector hunk should select selector.spec.ts: {selected:?}"
    );
    assert!(
        !selected.iter().any(|t| t.contains("unrelated.spec.ts")),
        "unrelated.spec.ts must not be selected: {selected:?}"
    );
}

#[test]
fn traces_lockfile_dependency_impact_matching_inline_diff_planning() {
    let dir = two_commit_hunks_repo();
    let root = dir.path();

    let output = Command::new(bin())
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
        .unwrap();
    let plan = plan_json(&output);

    assert!(
        !plan["fallback_triggered"].as_bool().unwrap(),
        "workspace package bump should be traceable, not fall back: {plan:?}"
    );
    let warnings = plan["warnings"].as_array().unwrap();
    assert!(
        !warnings.iter().any(|w| w["type"] == "lockfile-no-baseline"),
        "base/head streaming must resolve lockfile baselines: {warnings:?}"
    );
    let selected = selected_test_names(&plan);
    assert!(
        selected
            .iter()
            .any(|t| t.contains("packages/app/src/utils.test.ts")),
        "workspace package bump should trace to utils.test.ts: {selected:?}"
    );
}

// The patch exceeds 10 MB via many distinct lines (not one pathological
// line — that boundary is covered at the unit level in
// `git_diff::tests::stream_git_diff_rejects_a_pathologically_long_line`).
// Generated at runtime rather than committed: a 10+ MB fixture blob would
// bloat the repository for no benefit, so this is an intentional deviation
// from "fixtures live under test-cases/**" — do not "simplify" it into a
// committed file.
#[test]
fn a_patch_larger_than_ten_megabytes_completes_without_a_max_buffer_failure() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    setup_git_repo(root);

    const LINE_COUNT: usize = 220_000; // ~60 bytes/line * 2 (old+new) > 10 MB
    let mut content = String::new();
    for i in 0..LINE_COUNT {
        content.push_str(&format!("line number {i} before the change\n"));
    }
    std::fs::write(root.join("generated-large.txt"), &content).unwrap();
    git_commit_all(root, "base");

    let mut changed = String::new();
    for i in 0..LINE_COUNT {
        changed.push_str(&format!("line number {i} after the change\n"));
    }
    std::fs::write(root.join("generated-large.txt"), &changed).unwrap();
    git_commit_all(root, "head");

    let diff_size = git(&["diff", "HEAD~1...HEAD"], root).stdout.len();
    assert!(
        diff_size > 10 * 1024 * 1024,
        "fixture diff should exceed 10 MB, got {diff_size} bytes"
    );

    let output = Command::new(bin())
        .args([
            "tests",
            "plan",
            "--root",
            root.to_str().unwrap(),
            "--base",
            "HEAD~1",
            "--head",
            "HEAD",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn invalid_base_ref_reports_merge_base_unavailable_and_is_nonzero() {
    let dir = two_commit_hunks_repo();
    let root = dir.path();

    let output = Command::new(bin())
        .args([
            "tests",
            "plan",
            "--root",
            root.to_str().unwrap(),
            "--base",
            "not-a-real-ref",
            "--head",
            "HEAD",
            "--json",
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("git-merge-base-unavailable"),
        "expected stable diagnostic code in stderr: {stderr}"
    );
    // Never an empty confident plan: stdout must not be a valid, empty plan.
    assert!(stdout(&output).trim().is_empty());
}

#[test]
fn shallow_clone_missing_the_merge_base_reports_shallow_history_and_is_nonzero() {
    let origin_dir = tempfile::tempdir().unwrap();
    let origin = origin_dir.path();
    setup_git_repo(origin);
    std::fs::write(origin.join("f.txt"), "a\n").unwrap();
    git_commit_all(origin, "c2");
    git(&["checkout", "-q", "-b", "feature"], origin);
    std::fs::write(origin.join("f.txt"), "a\nb\n").unwrap();
    git_commit_all(origin, "c3-feature");

    let shallow_dir = tempfile::tempdir().unwrap();
    let shallow = shallow_dir.path();
    let clone = Command::new("git")
        .args([
            "clone",
            "--quiet",
            "--no-local",
            "--depth=1",
            "--branch",
            "feature",
            &format!("file://{}", origin.display()),
            shallow.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(clone.status.success());
    let fetch = git(
        &[
            "fetch",
            "--quiet",
            "--depth=1",
            "origin",
            "main:refs/remotes/origin/main",
        ],
        shallow,
    );
    assert!(fetch.status.success());

    let output = Command::new(bin())
        .args([
            "tests",
            "plan",
            "--root",
            shallow.to_str().unwrap(),
            "--base",
            "origin/main",
            "--head",
            "HEAD",
            "--json",
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("git-shallow-history"),
        "expected stable diagnostic code in stderr: {stderr}"
    );
    assert!(stdout(&output).trim().is_empty());
}

/// Three `.no-mistakes.yml` revisions: the initial fixture content, a `base`
/// commit, and a `head` commit, ending with the working tree detached at the
/// *pre-base* commit — matching neither `--base` nor `--head`'s config
/// content, like a CI runner checked out at a synthetic merge commit.
fn detached_config_change_repo() -> (tempfile::TempDir, String, String) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    copy_dir_all(&fixture_dir("hunks/initial"), root);
    setup_git_repo(root);
    let root_sha = stdout(&git(&["rev-parse", "HEAD"], root))
        .trim()
        .to_string();

    std::fs::copy(
        fixture_dir("hunks/config-base.yml"),
        root.join(".no-mistakes.yml"),
    )
    .unwrap();
    git_commit_all(root, "config-base");
    let base_sha = stdout(&git(&["rev-parse", "HEAD"], root))
        .trim()
        .to_string();

    std::fs::copy(
        fixture_dir("hunks/config-head.yml"),
        root.join(".no-mistakes.yml"),
    )
    .unwrap();
    git_commit_all(root, "config-head");
    let head_sha = stdout(&git(&["rev-parse", "HEAD"], root))
        .trim()
        .to_string();

    let checkout = git(&["checkout", "-q", &root_sha], root);
    assert!(checkout.status.success());

    (dir, base_sha, head_sha)
}

// Regression for a review finding on #587: automatic base/head streaming
// populates `diff_files`, which used to make `compare_changed_config` also
// run the diff-side hunk reconstruction (`sources_from_diff`) meant for
// explicit `--diff*` inputs. That path reads whatever `.no-mistakes.yml` is
// physically on disk and reconstructs the other side by applying hunks to
// it, which fails whenever the checkout is not exactly the base or head
// endpoint — even though `sources_from_git` (`git show <ref>:<path>`) already
// gives a checkout-independent comparison for the automatic path. The
// `sources_from_diff` failure is `?`-propagated out of `compare_changed_config`
// entirely, discarding the already-successful `sources_from_git` comparison;
// its caller (`framework_config_trigger`) then treats any error as "we could
// not read both revisions" and conservatively force-triggers the global
// config fallback for *every* framework, even one the config change never
// touched. The fixture config change here is Playwright-only
// (`testPlan.playwright.fullSuiteTriggers`), so a correct, precise comparison
// must leave a Vitest plan's `fallback_triggered` false.
#[test]
fn base_head_config_change_succeeds_when_checkout_matches_neither_endpoint() {
    let (dir, base_sha, head_sha) = detached_config_change_repo();
    let root = dir.path();

    let output = Command::new(bin())
        .args([
            "tests",
            "plan",
            "vitest",
            "--root",
            root.to_str().unwrap(),
            "--base",
            &base_sha,
            "--head",
            &head_sha,
            "--global-config-fallback",
            "true",
            "--json",
        ])
        .output()
        .unwrap();
    let plan = plan_json(&output);
    assert_eq!(
        plan["fallback_triggered"], false,
        "a Playwright-only config change must not force a Vitest full-suite \
         fallback, even when the checkout matches neither --base nor --head: {plan:?}"
    );
}
