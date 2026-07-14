use super::*;
use std::path::Path;
use std::process::Command;

fn nonexistent_config() -> std::path::PathBuf {
    std::path::PathBuf::from("nonexistent-config.yaml")
}

fn assert_no_fetch_root() -> std::path::PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/react-traits-config/assert-no-fetch/fixture"),
    )
}

fn initialize_git_fixture(root: &Path) {
    for args in [
        ["init", "-q", "--initial-branch=main"].as_slice(),
        ["add", "."].as_slice(),
    ] {
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
}

#[test]
fn run_check_with_facts_skips_when_assert_no_fetch_is_disabled() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/react-traits-components/nested/fixture");
    let facts = crate::codebase::check_facts::CheckFactMap::default();

    let findings = run_check_with_facts(
        &root,
        None,
        &["app/components/Child.tsx".to_string()],
        false,
        &facts,
    )
    .unwrap();

    assert!(findings.is_empty());
}

#[test]
fn run_check_reports_violations_when_assert_no_fetch_is_enabled() {
    let root = assert_no_fetch_root();
    let violations = run_check(&root, None, &[], true).unwrap();
    assert!(!violations.is_empty(), "expected fetch violations");
}

#[test]
fn check_enabled_returns_true_when_assert_no_fetch_is_enabled() {
    let root = assert_no_fetch_root();
    assert!(check_enabled(&root, None, true).unwrap());
}

#[test]
fn prepared_check_uses_already_loaded_unified_config() {
    let root = assert_no_fetch_root();
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();

    let prepared = prepare_check_from_loaded_config(&config, false);

    assert!(prepared.enabled());
}

#[test]
fn check_enabled_returns_false_when_assert_no_fetch_is_disabled() {
    // Fixture with no assert_no_fetch config
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/react-traits-components/nested/fixture");
    assert!(!check_enabled(&root, None, false).unwrap());
}

#[test]
fn run_check_returns_error_for_nonexistent_config_path() {
    let root = assert_no_fetch_root();
    let config = nonexistent_config();
    let err = run_check(&root, Some(&config), &[], false);
    assert!(err.is_err(), "expected error for nonexistent config");
}

#[test]
fn run_check_with_facts_returns_error_for_nonexistent_config_path() {
    let root = assert_no_fetch_root();
    let config = nonexistent_config();
    let facts = crate::codebase::check_facts::CheckFactMap::default();
    let err = run_check_with_facts(&root, Some(&config), &[], false, &facts);
    assert!(err.is_err(), "expected error for nonexistent config");
}

#[test]
fn check_enabled_returns_error_for_nonexistent_config_path() {
    let root = assert_no_fetch_root();
    let config = nonexistent_config();
    let err = check_enabled(&root, Some(&config), false);
    assert!(err.is_err(), "expected error for nonexistent config");
}

#[test]
fn run_check_with_facts_reports_violations_when_assert_no_fetch_is_enabled() {
    use crate::codebase::check_facts::{collect_check_facts, CheckFactPlan};

    let root = assert_no_fetch_root();
    let fetcher = root.join("app/components/Fetcher.tsx");
    let plan = CheckFactPlan {
        react: true,
        ..CheckFactPlan::default()
    };
    let facts = collect_check_facts(&root, vec![fetcher], plan);
    let violations = run_check_with_facts(&root, None, &[], true, &facts).unwrap();
    assert!(!violations.is_empty(), "expected fetch violations");
}

#[test]
fn prepared_check_uses_frozen_visible_files_after_source_is_removed() {
    use crate::codebase::check_facts::{collect_check_facts, CheckFactPlan};

    let dir = crate::test_support::materialize_gitignore_fixture("react-prepared-snapshot");
    initialize_git_fixture(dir.path());
    let root = crate::codebase::ts_resolver::normalize_path(dir.path());
    let visible = crate::codebase::ts_source::discover_visible_paths(&root);
    let fetcher = root.join("app/Fetcher.tsx");
    let ignored = root.join("app/Ignored.tsx");
    assert!(visible.contains(&fetcher));
    assert!(!visible.contains(&ignored));

    let facts = collect_check_facts(
        &root,
        visible,
        CheckFactPlan {
            react: true,
            ..CheckFactPlan::default()
        },
    );
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let prepared = prepare_check_from_loaded_config(&config, false);
    std::fs::remove_file(&fetcher).unwrap();

    let violations = run_check_with_prepared_facts(&root, &[], &facts, &prepared).unwrap();

    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, "assert-no-fetch");
    assert!(violations[0].file.ends_with("app/Fetcher.tsx"));
}
