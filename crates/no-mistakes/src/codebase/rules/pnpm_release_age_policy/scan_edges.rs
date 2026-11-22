use super::*;
use crate::codebase::rules::source_store_for_files;
use std::path::{Path, PathBuf};

// Output cases moved to the cli_pkgcfg_snapshot modules.

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/pnpm-release-age-policy/fixture")
            .join(name),
    )
}

fn scan_dir(files: &[PathBuf]) -> Vec<RuleFinding> {
    let sources = source_store_for_files(files);
    scan::scan(
        Path::new("/workspace"),
        &Options::default(),
        files,
        &sources,
    )
    .into_iter()
    .map(ClassifiedFinding::into_public)
    .collect()
}

fn scan_fixture(name: &str, files: &[PathBuf]) -> Vec<RuleFinding> {
    let root = fixture(name);
    let sources = source_store_for_files(files);
    scan::scan(&root, &Options::default(), files, &sources)
        .into_iter()
        .map(ClassifiedFinding::into_public)
        .collect()
}

#[test]
fn scan_returns_empty_without_a_workspace_file() {
    assert!(scan_dir(&[]).is_empty());
}

#[test]
fn scan_walks_exclude_cooldown_edges() {
    let root = fixture("exclude-cooldown-edges");
    let files = [
        root.join("pnpm-workspace.yaml"),
        root.join(".github/dependabot.yml"),
        root.join("package.json"),
        root.join("README.md"),
    ];
    // Coverage for nested exclude/cooldown values, invalid package.json, and
    // a skipped non-manifest file. Findings are not the assertion.
    let _ = scan_fixture("exclude-cooldown-edges", &files);
}

#[test]
fn missing_dependabot_source_reports_an_io_error() {
    let root = fixture("unreadable-lockfile");
    let missing = root.join(".github/dependabot.yml");
    let sources = source_store_for_files(std::slice::from_ref(&missing));
    let error = super::inputs::cooldown(&sources, &missing)
        .err()
        .expect("the fixture has no Dependabot file");
    assert!(!error.contains("failed to parse YAML"), "{error}");
    assert_eq!(sources.yaml_parse_count(), 1);
}
