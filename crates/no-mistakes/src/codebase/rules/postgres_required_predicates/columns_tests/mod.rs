use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

mod catalog;
mod options;

pub(super) fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-required-predicates/fixture")
            .join(name),
    )
}

pub(super) fn config_yaml(yaml: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(yaml).unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

pub(super) const COLUMNS: &str = r#"
sqlInclude: ["sql/**/*.sql"]
schemaCatalogPath: schema.json
partitionKeys: require
partitionKeyExemptions: [{table: bare, reason: legacy incomplete snapshot fixture}]
relations:
  - table: orders
    requireColumns: [account_id]
"#;

pub(super) fn run(dir: &str, yaml: &str, file: &str) -> Vec<RuleFinding> {
    let root = fixture(dir);
    let sql = root.join(file);
    check_with_files(&root, &config_yaml(yaml), std::slice::from_ref(&sql)).unwrap()
}

pub(super) fn messages(findings: &[RuleFinding]) -> String {
    findings
        .iter()
        .map(|finding| finding.message.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn invalid_examples_are_reported() {
    let kind = messages(&run("columns", COLUMNS, "sql/fail-kind.sql"));
    assert!(
        kind.contains("SELECT reads partitioned table events without constraining partition key column account_id"),
        "{kind}"
    );
    let join = messages(&run("columns", COLUMNS, "sql/fail-join.sql"));
    assert!(join.contains("partition key column account_id"), "{join}");
    let or_sql = messages(&run("columns", COLUMNS, "sql/fail-or.sql"));
    assert!(
        or_sql.contains("partition key column account_id"),
        "{or_sql}"
    );
    let delete = messages(&run("columns", COLUMNS, "sql/fail-delete.sql"));
    assert!(
        delete.contains("DELETE on orders does not constrain required column account_id"),
        "{delete}"
    );
    let self_join = run("columns", COLUMNS, "sql/fail-self.sql");
    assert_eq!(self_join.len(), 1, "{self_join:?}");
    assert!(self_join[0]
        .message
        .contains("SELECT on orders does not constrain required column account_id"));
    let insert = messages(&run("columns", COLUMNS, "sql/fail-insert.sql"));
    assert!(
        insert.contains("INSERT … SELECT reads partitioned table events"),
        "{insert}"
    );
    let nulls = messages(&run("columns", COLUMNS, "sql/fail-null.sql"));
    assert!(nulls.contains("partition key column account_id"), "{nulls}");
}

#[test]
fn valid_examples_pass() {
    for file in [
        "sql/pass-eq.sql",
        "sql/pass-any.sql",
        "sql/pass-join.sql",
        "sql/pass-or.sql",
        "sql/pass-update.sql",
        "sql/pass-in.sql",
        "sql/pass-cte.sql",
        "sql/pass-qualified.sql",
        "sql/pass-unqualified.sql",
        "sql/pass-between.sql",
        "sql/pass-bare.sql",
        "sql/pass-unknown.sql",
    ] {
        let findings = run("columns", COLUMNS, file);
        assert!(findings.is_empty(), "{file} {findings:?}");
    }
}

#[test]
fn an_unknown_peer_blocks_an_unqualified_column() {
    let findings = run("columns", COLUMNS, "sql/fail-unknown-peer.sql");
    let body = messages(&findings);
    assert!(body.contains("partition key column account_id"), "{body}");
}

#[test]
fn ambiguous_unqualified_columns_are_not_resolved() {
    let findings = run("columns", COLUMNS, "sql/fail-ambiguous.sql");
    let body = messages(&findings);
    assert!(body.contains("partition key column account_id"), "{body}");
    assert!(
        body.contains("SELECT on orders does not constrain required column account_id"),
        "{body}"
    );
}
