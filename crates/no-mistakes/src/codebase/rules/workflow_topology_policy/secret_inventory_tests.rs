use super::*;
use crate::codebase::ci_workflows::PreparedWorkflowTopologyInputs;
use crate::codebase::ts_source::VisiblePathSnapshot;

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root(RULE_ID)
            .join("secret-inventory")
            .join(name),
    )
}

fn run(name: &str, file: &str) -> Result<Vec<RuleFinding>> {
    let root = fixture(name);
    let path = root.join(file);
    let config = crate::config::v2::load_v2_config(&root, Some(&path))?;
    let snapshot = VisiblePathSnapshot::new(&root);
    let sources = snapshot.source_store_for(&root);
    let inputs =
        PreparedWorkflowTopologyInputs::for_config(&root, &config, &snapshot, &sources).unwrap();
    check_prepared(
        &root,
        &config,
        &snapshot.paths_for(&root),
        &sources,
        &inputs,
        Some(&path),
    )
}

#[test]
fn declared_inventory_reports_real_secret_use_locations() {
    let findings = run("repository", ".no-mistakes.yml").unwrap();
    let lines: Vec<_> = findings
        .iter()
        .map(|finding| {
            assert_eq!(finding.file, ".github/workflows/check.yml");
            assert!(
                finding.message.starts_with("undeclared secret"),
                "{finding:?}"
            );
            finding.line
        })
        .collect();
    assert_eq!(lines, [8, 10, 13]);
}

#[test]
fn omitted_and_null_inventory_preserve_disabled_behavior() {
    for config in ["disabled.yml", "null.yml"] {
        assert!(run("repository", config).unwrap().is_empty());
    }
}

#[test]
fn empty_inventory_is_strict_and_builtin_exemptions_are_configurable() {
    assert_eq!(run("repository", "empty.yml").unwrap().len(), 5);
    let findings = run("repository", "strict-builtin.yml").unwrap();
    assert_eq!(findings.len(), 4);
    assert!(findings
        .iter()
        .any(|finding| finding.line == 7 && finding.message.contains("GITHUB_TOKEN")));
}

#[test]
fn inventory_rejects_case_insensitive_duplicate_names() {
    let error = run("repository", "duplicate.yml").unwrap_err().to_string();
    assert!(error.contains("duplicate secretInventory name"), "{error}");
}

#[test]
fn workflow_level_references_consume_repository_inventory_with_dependency_edges() {
    assert!(run("top-level", ".no-mistakes.yml").unwrap().is_empty());
}

#[test]
fn hyphenated_ids_do_not_reference_secrets_or_block_unused_reporting() {
    let findings = run("hyphenated-contexts", ".no-mistakes.yml").unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, ".no-mistakes.yml");
    assert!(findings[0]
        .message
        .contains("unused declared secret UNUSED in repository"));
}

#[path = "secret_inventory_environments_tests.rs"]
mod environments;

#[path = "secret_inventory_reusable_tests.rs"]
mod reusable;

#[path = "secret_inventory_incomplete_tests.rs"]
mod incomplete;

#[path = "secret_inventory_validation_tests.rs"]
mod validation;

#[path = "secret_inventory_failure_tests.rs"]
mod failure;

#[path = "secret_inventory_relative_config_tests.rs"]
mod relative_config;
