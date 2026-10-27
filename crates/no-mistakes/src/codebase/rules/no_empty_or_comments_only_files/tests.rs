use super::tests_support::check_file;
use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::Path;

fn config_with_rule(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

fn fixture(path: &str) -> std::path::PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/no-empty-or-comments-only-files/fixture")
            .join(path),
    )
}

fn case_root(name: &str) -> std::path::PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root("no-empty-or-comments-only-files")
            .join("cases")
            .join(name),
    )
}

// Output cases moved to cli_rra_snapshot_no_empty_or_comments_only_files.
// Keep sort order, default extensions, and missing-file targets.

#[test]
fn sort_fixture_findings_are_sorted() {
    let root = fixture("sort");
    let config_path = root.join(".no-mistakes.yml");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap();
    let files: Vec<_> = findings
        .iter()
        .map(|finding| finding.file.as_str())
        .collect();
    assert_eq!(files, vec!["a.ts", "b.ts"]);
}

#[test]
fn unreadable_file_reports_unanalyzable_input() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("missing.ts");
    let findings = check_file(&path, tmp.path());
    assert_eq!(findings[0].target.as_deref(), Some("unanalyzable"));
}

#[test]
fn default_extensions_used_when_extensions_empty() {
    // When no extensions are configured, effective_extensions returns DEFAULT_EXTENSIONS.
    // A .ts file with only a comment should still be flagged.
    let root = case_root("default-extensions-used-when-extensions-empty");
    let path = root.join("only_comment.ts");
    let config = config_with_rule("{}"); // empty options → default extensions
    let findings = check_with_files(&root, &config, &[path]).unwrap();
    assert_eq!(
        findings.len(),
        1,
        "default extensions should catch .ts files"
    );
}
