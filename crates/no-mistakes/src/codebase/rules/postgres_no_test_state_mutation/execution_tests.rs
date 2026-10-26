use super::*;
use crate::config::v2::schema::{RuleDef, RuleScope};

#[test]
fn explain_reports_only_analyzed_sql() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-no-test-state-mutation/fixture"),
    );
    let config = NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.into(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(
                "testGlobs: ['tests/**/*.test.ts']\nimportSpecifier: '@example/db'",
            )
            .unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let file = root.join("tests/explain.test.ts");
    let findings = check_with_files(&root, &config, &[file]).unwrap();
    let lines: Vec<_> = findings.iter().map(|finding| finding.line).collect();
    assert_eq!(lines, [8, 9, 10], "{findings:#?}");
    assert!(findings
        .iter()
        .all(|finding| finding.target.as_deref() == Some("pg_sleep")));
}
