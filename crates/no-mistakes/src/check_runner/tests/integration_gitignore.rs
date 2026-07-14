use super::super::{prepared, run_all};
use std::path::{Path, PathBuf};
use std::process::Command;

fn fixture() -> tempfile::TempDir {
    let dir = crate::test_support::materialize_gitignore_fixture("integration-aggregate");
    run_git(dir.path(), &["init", "-q", "--initial-branch=main"]);
    run_git(dir.path(), &["add", "."]);
    dir
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
