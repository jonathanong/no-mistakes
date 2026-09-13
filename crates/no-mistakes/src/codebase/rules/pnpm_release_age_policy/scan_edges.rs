use super::*;
use crate::codebase::rules::source_store_for_files;
use std::path::{Path, PathBuf};

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
}

fn scan_fixture(name: &str, files: &[PathBuf]) -> Vec<RuleFinding> {
    let root = fixture(name);
    let sources = source_store_for_files(files);
    scan::scan(&root, &Options::default(), files, &sources)
}

#[test]
fn scan_returns_empty_without_workspace_or_for_non_mapping_yaml() {
    assert!(scan_dir(&[]).is_empty());
    let root = fixture("non-mapping-scan");
    let files = [root.join("pnpm-workspace.yaml")];
    assert!(scan_fixture("non-mapping-scan", &files).is_empty());
}

#[test]
fn scan_reports_invalid_workspace_yaml() {
    let root = fixture("invalid-workspace-yaml");
    let files = [root.join("pnpm-workspace.yaml")];
    let findings = scan_fixture("invalid-workspace-yaml", &files);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("failed to parse YAML")),
        "{findings:?}"
    );
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
