use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-generated-column-predicates/fixture")
            .join(name),
    )
}

fn config_yaml(yaml: &str) -> NoMistakesConfig {
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

const SQL: &str = "sqlInclude: [\"sql/**/*.sql\"]\n";

fn messages(root: &str, yaml: &str) -> Vec<String> {
    let root = fixture(root);
    let mut files = Vec::new();
    for entry in std::fs::read_dir(root.join("sql")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("sql") {
            files.push(path);
        }
    }
    files.sort();
    check_with_files(&root, &config_yaml(yaml), &files)
        .unwrap()
        .into_iter()
        .map(|finding| finding.message)
        .collect()
}

#[test]
fn invalid_examples_are_reported() {
    let body = messages("fail", SQL).join("\n");
    assert!(
        body.contains(
            "WHERE filters orders.created_at, which is generated from uuid_extract_timestamp(id)"
        ),
        "{body}"
    );
    assert!(
        body.contains("ORDER BY orders.created_at sorts by a column generated from id"),
        "{body}"
    );
    assert!(
        body.contains(
            "WHERE filters orders.created_at, which is generated from uuid_extract_timestamp(id)"
        ),
        "{body}"
    );
}

#[test]
fn valid_examples_are_quiet() {
    let body = messages("pass", SQL);
    assert!(body.is_empty(), "{body:?}");
}

#[test]
fn stored_columns_are_tracked_and_non_pk_arguments_are_not() {
    let body = messages("edges", SQL).join("\n");
    assert!(body.contains("WHERE filters orders.stored_at"), "{body}");
    assert!(body.contains("WHERE filters orders.virtual_at"), "{body}");
    assert!(
        body.contains("JOIN ON compares orders.created_at"),
        "{body}"
    );
    assert!(!body.contains("note_at"), "{body}");
    assert!(!body.contains("ambiguous"), "{body}");
    let off = messages(
        "edges",
        "sqlInclude: [\"sql/**/*.sql\"]\nrequireArgumentIsPrimaryKey: false\n",
    )
    .join("\n");
    assert!(off.contains("note_at"), "{off}");
}

#[test]
fn each_clause_can_be_switched_off() {
    let where_only =
        messages("fail", "sqlInclude: [\"sql/**/*.sql\"]\nclauses: [where]\n").join("\n");
    assert!(where_only.contains("WHERE filters"), "{where_only}");
    assert!(!where_only.contains("ORDER BY"), "{where_only}");
    let order_only = messages(
        "fail",
        "sqlInclude: [\"sql/**/*.sql\"]\nclauses: [order-by]\n",
    )
    .join("\n");
    assert!(order_only.contains("ORDER BY"), "{order_only}");
    assert!(!order_only.contains("WHERE filters"), "{order_only}");
    let join_only =
        messages("edges", "sqlInclude: [\"sql/**/*.sql\"]\nclauses: [join]\n").join("\n");
    assert!(join_only.contains("JOIN ON"), "{join_only}");
    assert!(!join_only.contains("WHERE filters"), "{join_only}");
}

#[test]
fn stale_extra_generated_columns_are_reported() {
    let body = messages(
        "fail",
        "sqlInclude: [\"sql/**/*.sql\"]\nextraGeneratedColumns:\n  - {table: orders, column: created_at, sourceColumn: id}\n",
    )
    .join("\n");
    assert!(
        body.contains(
            "stale postgres-generated-column-predicates extraGeneratedColumns entry: orders.created_at"
        ),
        "{body}"
    );
}

#[test]
fn repeated_scans_match() {
    assert_eq!(messages("fail", SQL), messages("fail", SQL));
}

#[test]
fn suppression_directives_hide_predicates() {
    let root = fixture("suppress");
    let files: Vec<PathBuf> = ["schema.sql", "next-line.sql", "line.sql", "file.sql"]
        .into_iter()
        .map(|name| root.join("sql").join(name))
        .collect();
    let mut findings = check_with_files(&root, &config_yaml(SQL), &files).unwrap();
    assert_eq!(findings.len(), 3, "{findings:?}");
    let sources = super::super::source_store_for_files(&files);
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty(), "{findings:?}");
}
