use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

const BASE: &str = "importSpecifier: '@example/db'\nrelations:\n  - table: topics\n    require: [\"parent_id IS NOT NULL\"]\n";
const TRUSTED: &str = "importSpecifier: '@example/db'\ntrustedSqlTags:\n  - {module: '@example/db', name: sql}\nrelations:\n  - table: topics\n    require: [\"parent_id IS NOT NULL\"]\n";

fn run(file: &str, options: &str, suppress: bool) -> Vec<RuleFinding> {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-required-predicates/fixture/fail-tagged"),
    );
    let config = NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(options).unwrap(),
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

fn anchors(findings: &[RuleFinding]) -> Vec<(Option<String>, usize)> {
    findings
        .iter()
        .map(|finding| (finding.target.clone(), finding.line))
        .collect()
}

#[test]
fn configured_named_tag_anchors_at_the_relation_line() {
    let findings = run("src/imported-tag.ts", TRUSTED, false);
    assert_eq!(anchors(&findings), vec![(Some("topics".into()), 10)]);
}

#[test]
fn same_tag_without_the_option_stays_unanalyzable() {
    let findings = run("src/imported-tag.ts", BASE, false);
    assert_eq!(anchors(&findings), vec![(Some("unanalyzable".into()), 8)]);
}

#[test]
fn in_sql_directive_suppresses_only_after_the_tag_is_trusted() {
    assert_eq!(
        anchors(&run("src/imported-tag-suppressed.ts", BASE, true)),
        vec![(Some("unanalyzable".into()), 7)]
    );
    assert_eq!(
        anchors(&run("src/imported-tag-suppressed.ts", TRUSTED, false)),
        vec![(Some("topics".into()), 10)]
    );
    assert!(run("src/imported-tag-suppressed.ts", TRUSTED, true).is_empty());
}

#[test]
fn same_named_tag_from_another_module_stays_untrusted() {
    let findings = run("src/imported-tag-other-module.ts", TRUSTED, false);
    assert_eq!(anchors(&findings), vec![(Some("unanalyzable".into()), 7)]);
}

#[test]
fn subpath_is_analyzed_and_a_sibling_prefix_is_not() {
    assert_eq!(
        anchors(&run("src/imported-tag-subpath.ts", TRUSTED, false)),
        vec![(Some("topics".into()), 8)]
    );
    assert_eq!(
        anchors(&run("src/imported-tag-sibling.ts", TRUSTED, false)),
        vec![(Some("unanalyzable".into()), 6)]
    );
}
