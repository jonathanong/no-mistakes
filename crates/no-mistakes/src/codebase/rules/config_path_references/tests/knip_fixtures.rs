use super::*;
use crate::codebase::rules::run_filesystem_rules;

fn knip_fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/config-path-references")
            .join(name),
    )
}

fn knip_files(root: &Path) -> Vec<PathBuf> {
    listed(
        root,
        &[
            ".no-mistakes.yml",
            "knip.json",
            "backend/dependency-cruiser-rules/__tests__/fixtures/build-insert-query/direct-import.mts",
        ],
    )
}

#[test]
fn knip_workspace_entries_under_fixtures_exist_when_listed() {
    let root = knip_fixture("knip-workspace-fixtures-pass");
    let mut files = knip_files(&root);
    files.push(root.join(
        "backend/dependency-cruiser-rules/__tests__/fixtures/build-insert-query/aliased-import.mts",
    ));
    let findings = check_with_files(&root, &preset_config(&root), &files).unwrap();
    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn knip_workspace_entries_under_fixtures_report_missing_literals() {
    let root = knip_fixture("knip-workspace-fixtures-fail");
    let findings = check_with_files(&root, &preset_config(&root), &knip_files(&root)).unwrap();
    assert!(
        findings.iter().any(|finding| {
            finding
                .message
                .contains("backend/dependency-cruiser-rules/__tests__/fixtures/build-insert-query/missing.mts")
        }),
        "{findings:?}"
    );
}

fn git_tracked_knip_root(name: &str) -> tempfile::TempDir {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/config-path-references")
        .join(name);
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    fixture
}

#[test]
fn knip_workspace_entries_under_fixtures_pass_from_tracked_inventory() {
    let fixture = git_tracked_knip_root("knip-workspace-fixtures-pass");
    let findings = run_filesystem_rules(
        fixture.path(),
        Some(&fixture.path().join(".no-mistakes.yml")),
    )
    .unwrap();
    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn knip_workspace_entries_under_fixtures_fail_from_tracked_inventory() {
    let fixture = git_tracked_knip_root("knip-workspace-fixtures-fail");
    let findings = run_filesystem_rules(
        fixture.path(),
        Some(&fixture.path().join(".no-mistakes.yml")),
    )
    .unwrap();
    assert!(
        findings.iter().any(|finding| {
            finding.rule == RULE_ID
                && finding.message.contains(
                    "backend/dependency-cruiser-rules/__tests__/fixtures/build-insert-query/missing.mts",
                )
        }),
        "{findings:?}"
    );
}

#[test]
fn knip_workspace_entries_under_fixtures_ignore_untracked_files() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/config-path-references/knip-workspace-fixtures-pass");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_force(fixture.path(), &[".no-mistakes.yml", "knip.json"]);
    let findings = run_filesystem_rules(
        fixture.path(),
        Some(&fixture.path().join(".no-mistakes.yml")),
    )
    .unwrap();
    assert!(
        findings.iter().any(|finding| {
            finding.rule == RULE_ID
                && finding.message.contains(
                    "backend/dependency-cruiser-rules/__tests__/fixtures/build-insert-query/direct-import.mts",
                )
        }),
        "{findings:?}"
    );
}
