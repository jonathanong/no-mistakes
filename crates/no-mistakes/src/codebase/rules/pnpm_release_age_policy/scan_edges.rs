use super::*;
use crate::codebase::rules::source_store_for_files;
use std::fs;
use std::path::{Path, PathBuf};

fn scan_dir(files: &[PathBuf]) -> Vec<RuleFinding> {
    let sources = source_store_for_files(files);
    scan::scan(
        Path::new("/workspace"),
        &Options::default(),
        files,
        &sources,
    )
}

#[test]
fn scan_returns_empty_without_workspace_or_for_non_mapping_yaml() {
    assert!(scan_dir(&[]).is_empty());
    let root = tempfile::tempdir().unwrap();
    let workspace = root.path().join("pnpm-workspace.yaml");
    fs::write(&workspace, "- not-a-mapping\n").unwrap();
    let files = [workspace];
    let sources = source_store_for_files(&files);
    assert!(scan::scan(root.path(), &Options::default(), &files, &sources).is_empty());
}

#[test]
fn scan_reports_invalid_workspace_yaml_and_walks_exclude_cooldown_edges() {
    let root = tempfile::tempdir().unwrap();
    let workspace = root.path().join("pnpm-workspace.yaml");
    fs::write(&workspace, "minimumReleaseAgeExclude: [ok, {nested: 1}]\n:").unwrap();
    let files = [workspace.clone()];
    let sources = source_store_for_files(&files);
    let findings = scan::scan(root.path(), &Options::default(), &files, &sources);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("failed to parse YAML")),
        "{findings:?}"
    );

    fs::write(&workspace, "minimumReleaseAgeExclude: [ok, {nested: 1}]\n").unwrap();
    let dependabot = root.path().join(".github/dependabot.yml");
    fs::create_dir_all(dependabot.parent().unwrap()).unwrap();
    fs::write(
        &dependabot,
        "updates:\n  - package-ecosystem: npm\n    directory: /\n    cooldown:\n      exclude: [ok, {nested: 1}]\n",
    )
    .unwrap();
    let package = root.path().join("package.json");
    fs::write(&package, "{ not json").unwrap();
    let other = root.path().join("README.md");
    fs::write(&other, "skip\n").unwrap();
    let files = [workspace, dependabot, package, other];
    let sources = source_store_for_files(&files);
    let findings = scan::scan(root.path(), &Options::default(), &files, &sources);
    let _ = findings;
}
