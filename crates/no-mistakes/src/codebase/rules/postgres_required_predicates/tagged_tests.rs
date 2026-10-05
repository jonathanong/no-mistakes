use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

// Saved fixtures: multi-line `sql`-tagged templates whose FROM is on a later
// line than the call and template start.
fn run(file: &str, suppress: bool) -> Vec<RuleFinding> {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-required-predicates/fixture/fail-tagged"),
    );
    let config = NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(
                "importSpecifier: '@example/db'\nrelations:\n  - table: topics\n    require: [\"parent_id IS NOT NULL\"]\n  - table: orders\n    requireColumns: [account_id]",
            )
            .unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let files = [root.join(file)];
    let mut findings = check_with_files(&root, &config, &files).unwrap();
    if suppress {
        let sources = super::super::source_store_for_files(&files);
        super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    }
    findings
}

#[test]
fn tagged_template_findings_anchor_at_the_relation_line() {
    let findings = run("src/tagged.ts", false);
    let lines: Vec<_> = findings
        .iter()
        .map(|f| (f.target.clone(), f.line))
        .collect();
    assert_eq!(
        lines,
        [(Some("topics".into()), 9), (Some("orders".into()), 17)],
        "{findings:?}"
    );
}

#[test]
fn in_sql_directive_above_the_from_line_suppresses_tagged_findings() {
    assert_eq!(run("src/tagged-suppressed.ts", false).len(), 2);
    let findings = run("src/tagged-suppressed.ts", true);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn imported_tag_is_unrecoverable_and_anchors_at_the_call() {
    let findings = run("src/imported-tag.ts", false);
    let lines: Vec<_> = findings
        .iter()
        .map(|f| (f.target.clone(), f.line))
        .collect();
    assert_eq!(lines, [(Some("unanalyzable".into()), 8)], "{findings:?}");
}
