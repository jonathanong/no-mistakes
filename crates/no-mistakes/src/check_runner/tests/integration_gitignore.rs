use super::super::{prepared, run_all};
use std::path::{Path, PathBuf};
use std::process::Command;

fn fixture() -> tempfile::TempDir {
    let dir = crate::test_support::materialize_gitignore_fixture("integration-aggregate");
    run_git(dir.path(), &["init", "-q", "--initial-branch=main"]);
    run_git(dir.path(), &["add", "."]);
    dir
}

fn banned_paths_fixture() -> tempfile::TempDir {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/gitignore/banned-paths-tracked-only");
    let dir = crate::test_support::materialize_saved_fixture(&source);
    run_git(dir.path(), &["init", "-q", "--initial-branch=main"]);
    run_git(dir.path(), &["add", ".no-mistakes.yml", "tracked.patch"]);
    dir
}

fn install_banned_paths_ignore(root: &Path) {
    std::fs::rename(
        root.join("gitignore-after.fixture"),
        root.join(".gitignore"),
    )
    .unwrap();
    run_git(root, &["add", ".gitignore"]);
}

fn banned_path_files(root: &Path) -> Vec<String> {
    run_all(root.to_path_buf(), None, None)
        .unwrap()
        .rules
        .into_iter()
        .filter(|finding| finding.rule == no_mistakes::codebase::rules::BANNED_PATHS)
        .map(|finding| finding.file)
        .collect()
}

fn run_git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .env_remove("GIT_DIR")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn integration_suites(
    root: &Path,
    config: Option<PathBuf>,
    tsconfig: Option<PathBuf>,
) -> Vec<String> {
    run_all(root.to_path_buf(), config, tsconfig)
        .unwrap()
        .integration
        .into_iter()
        .map(|finding| finding.suite)
        .collect()
}

#[test]
fn aggregate_integration_skips_ignored_automatic_runner_configs() {
    let dir = fixture();

    let suites = integration_suites(dir.path(), None, None);

    assert_eq!(suites, vec!["auto-playwright.unit", "auto-vitest.unit"]);
}

#[test]
fn aggregate_integration_honors_explicit_ignored_runner_configs() {
    let dir = fixture();
    let config = dir.path().join("explicit.no-mistakes.yml");

    let suites = integration_suites(dir.path(), Some(config), None);

    assert_eq!(
        suites,
        vec!["explicit-playwright.unit", "explicit-vitest.unit"]
    );
}

#[test]
fn banned_paths_uses_the_git_index_and_keeps_tracked_ignored_files() {
    let dir = banned_paths_fixture();

    assert_eq!(banned_path_files(dir.path()), ["tracked.patch"]);

    // Install the ignore rule only after tracking the patch. This order is the
    // invariant: a later ignore must not hide a path already present in the index.
    install_banned_paths_ignore(dir.path());

    assert_eq!(banned_path_files(dir.path()), ["tracked.patch"]);
}

#[test]
fn banned_paths_uses_ignore_aware_files_outside_git() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/gitignore/banned-paths-tracked-only");
    let dir = crate::test_support::materialize_saved_fixture(&source);
    std::fs::rename(
        dir.path().join("gitignore-after.fixture"),
        dir.path().join(".gitignore"),
    )
    .unwrap();

    assert_eq!(banned_path_files(dir.path()), ["untracked-visible.patch"]);
}

#[test]
fn aggregate_integration_skips_ignored_automatic_tsconfig_but_honors_explicit_path() {
    let dir = fixture();

    let automatic = prepared::prepare(dir.path(), None, None).unwrap();
    assert!(automatic.tsconfig.paths.is_empty());
    assert!(automatic.tsconfig.base_url.is_none());

    let explicit_path = PathBuf::from("tsconfig.json");
    let explicit = prepared::prepare(dir.path(), None, Some(&explicit_path)).unwrap();
    assert!(!explicit.tsconfig.paths.is_empty());
    assert_eq!(explicit.tsconfig.base_url.as_deref(), Some(dir.path()));

    let suites = integration_suites(dir.path(), None, Some(explicit_path));
    assert_eq!(suites, vec!["auto-playwright.unit", "auto-vitest.unit"]);
}
