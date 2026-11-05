use super::{check_with_files, RULE_ID};
use crate::codebase::rules::RuleFinding;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

pub(super) const IDS: &str = "schemaCatalogPath: schema.json\nidKeyPattern: '(^id$|Id$|_id$)'\nkeyColumnMap: {accountId: account_id}\nsqlInclude: ['db/**/*.sql', 'suppress/**/*.sql']\n";

pub(super) fn fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(&crate::test_support::rule_fixture_root(
        "postgres-json-relation-ids",
    ))
}

pub(super) fn config(yaml: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.into(),
            scope: Some(RuleScope::Repository),
            options: crate::codebase::postgres::tests::fixture_rule_options(yaml),
            ..Default::default()
        }],
        ..Default::default()
    }
}

pub(super) fn run(yaml: &str, files: &[&str]) -> Vec<RuleFinding> {
    let root = fixture();
    let files = files.iter().map(|file| root.join(file)).collect::<Vec<_>>();
    check_with_files(&root, &config(yaml), &files).unwrap()
}

#[test]
fn stale_fingerprint_allowance_requires_catalog_in_scope() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/unused-allow-other"),
    );
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let files =
        crate::codebase::ts_source::discover_files(&root, &config.filesystem.skip_directories);
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].rule, "unused-allow-entry");
    let findings = check_with_files(&root, &config, &[]).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}
