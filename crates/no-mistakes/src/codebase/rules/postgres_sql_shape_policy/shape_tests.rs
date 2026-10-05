use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

pub(super) fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-sql-shape-policy/fixture")
            .join(name),
    )
}

pub(super) fn config_yaml(yaml: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: crate::codebase::postgres::tests::fixture_rule_options(yaml),
            ..Default::default()
        }],
        ..Default::default()
    }
}

pub(super) const BANNED: &str = r#"
sqlInclude: ["sql/**/*.sql"]
bannedShapes:
  - correlated-exists-set-operation
  - not-in-subquery
  - count-for-existence
"#;

fn messages(root: &str, file: &str) -> String {
    let root = fixture(root);
    let sql = root.join(file);
    check_with_files(&root, &config_yaml(BANNED), std::slice::from_ref(&sql))
        .unwrap()
        .into_iter()
        .map(|finding| finding.message)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn new_shapes_are_off_by_default() {
    let compiled = compile_options(&Options {
        executor_names: Some(Vec::new()),
        ..Default::default()
    })
    .unwrap();
    assert!(compiled.shapes.correlated_exists_set_operation);
    assert!(!compiled.shapes.not_in_subquery);
    assert!(!compiled.shapes.count_for_existence);
    let root = fixture("pass-default-shapes");
    let sql = root.join("sql/001.sql");
    let findings = check_with_files(
        &root,
        &config_yaml("sqlInclude: [\"sql/**/*.sql\"]\n"),
        std::slice::from_ref(&sql),
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn invalid_examples_are_reported_when_opted_in() {
    let not_in = messages("fail-shapes", "sql/not-in.sql");
    assert!(
        not_in.contains("NOT IN (SELECT …) returns no rows when the subquery yields a NULL"),
        "{not_in}"
    );
    let count = messages("fail-shapes", "sql/count.sql");
    assert!(
        count.contains("COUNT(*) compared with 0/1 counts every matching row"),
        "{count}"
    );
    let root = fixture("pass-shapes");
    for name in [
        "sql/001.sql",
        "sql/list.sql",
        "sql/having.sql",
        "sql/threshold.sql",
        "sql/plain-count.sql",
    ] {
        let file = root.join(name);
        let findings =
            check_with_files(&root, &config_yaml(BANNED), std::slice::from_ref(&file)).unwrap();
        assert!(findings.is_empty(), "{name} {findings:?}");
    }
}

#[test]
fn unknown_shape_is_still_a_config_error() {
    let Err(error) = compile_options(&Options {
        executor_names: Some(Vec::new()),
        banned_shapes: vec!["not-in-subquery".into(), "sideways".into()],
        ..Default::default()
    }) else {
        panic!("compiled");
    };
    assert!(
        error
            .to_string()
            .contains("unknown bannedShapes value `sideways`"),
        "{error}"
    );
    assert!(
        compile_options(&Options {
            executor_names: Some(Vec::new()),
            banned_shapes: vec!["not-in-subquery".into(), "count-for-existence".into()],
            ..Default::default()
        })
        .unwrap()
        .shapes
        .not_in_subquery
    );
}

#[test]
fn suppression_directives_hide_the_new_shapes() {
    let root = fixture("suppress-shapes");
    let config = config_yaml(BANNED);
    for name in ["sql/next-line.sql", "sql/line.sql", "sql/file.sql"] {
        let file = root.join(name);
        let mut findings = check_with_files(&root, &config, std::slice::from_ref(&file)).unwrap();
        assert!(!findings.is_empty(), "{name} {findings:?}");
        let sources = super::super::source_store_for_files(std::slice::from_ref(&file));
        super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
        assert!(findings.is_empty(), "{name} {findings:?}");
    }
}

#[test]
fn disabling_the_exists_shape_skips_that_finding() {
    let root = fixture("fail");
    let sql = root.join("sql/001.sql");
    let findings = check_with_files(
        &root,
        &config_yaml("sqlInclude: [\"sql/**/*.sql\"]\nbannedShapes: [not-in-subquery]\n"),
        std::slice::from_ref(&sql),
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn identical_shapes_on_one_line_are_reported_once() {
    let root = fixture("fail-shapes");
    let sql = root.join("sql/same-line.sql");
    let findings =
        check_with_files(&root, &config_yaml(BANNED), std::slice::from_ref(&sql)).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn repeated_shape_scans_match() {
    let root = fixture("fail-shapes");
    let sql = root.join("sql/not-in.sql");
    let config = config_yaml(BANNED);
    let first = check_with_files(&root, &config, std::slice::from_ref(&sql)).unwrap();
    let second = check_with_files(&root, &config, std::slice::from_ref(&sql)).unwrap();
    assert_eq!(first, second);
}
