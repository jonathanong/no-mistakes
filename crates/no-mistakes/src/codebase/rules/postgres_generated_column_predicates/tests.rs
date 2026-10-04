use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

pub(super) fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-generated-column-predicates/fixture")
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

const SQL: &str = "sqlInclude: [\"sql/**/*.sql\"]\n";

fn messages(root: &str, yaml: &str, files: &[&str]) -> Vec<String> {
    let root = fixture(root);
    let paths: Vec<PathBuf> = files.iter().map(|file| root.join(file)).collect();
    check_with_files(&root, &config_yaml(yaml), &paths)
        .unwrap()
        .into_iter()
        .map(|finding| finding.message)
        .collect()
}

#[test]
fn invalid_examples_are_reported() {
    let body = messages(
        "fail",
        SQL,
        &[
            "sql/schema.sql",
            "sql/where.sql",
            "sql/order.sql",
            "sql/between.sql",
        ],
    )
    .join("\n");
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
fn standalone_table_order_checks_generated_columns() {
    let found = messages("fail", SQL, &["sql/schema.sql", "sql/standalone-table.sql"]);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("ORDER BY orders.created_at"), "{found:?}");
}

#[test]
fn valid_examples_are_quiet() {
    let body = messages(
        "pass",
        SQL,
        &[
            "sql/schema.sql",
            "sql/order-id.sql",
            "sql/where-id.sql",
            "sql/isnull.sql",
            "sql/trunc.sql",
            "sql/invoices.sql",
        ],
    );
    assert!(body.is_empty(), "{body:?}");
}

#[test]
fn stored_columns_are_tracked_and_non_pk_arguments_are_not() {
    let edges = [
        "sql/schema.sql",
        "sql/stored.sql",
        "sql/join.sql",
        "sql/note.sql",
        "sql/ambiguous.sql",
        "sql/unique.sql",
        "sql/virtual.sql",
    ];
    let body = messages("edges", SQL, &edges).join("\n");
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
        &edges,
    )
    .join("\n");
    assert!(off.contains("note_at"), "{off}");
}

#[test]
fn each_clause_can_be_switched_off() {
    let fail = [
        "sql/schema.sql",
        "sql/where.sql",
        "sql/order.sql",
        "sql/between.sql",
    ];
    let where_only = messages(
        "fail",
        "sqlInclude: [\"sql/**/*.sql\"]\nclauses: [where]\n",
        &fail,
    )
    .join("\n");
    assert!(where_only.contains("WHERE filters"), "{where_only}");
    assert!(!where_only.contains("ORDER BY"), "{where_only}");
    let order_only = messages(
        "fail",
        "sqlInclude: [\"sql/**/*.sql\"]\nclauses: [order-by]\n",
        &fail,
    )
    .join("\n");
    assert!(order_only.contains("ORDER BY"), "{order_only}");
    assert!(!order_only.contains("WHERE filters"), "{order_only}");
    let join_only = messages(
        "edges",
        "sqlInclude: [\"sql/**/*.sql\"]\nclauses: [join]\n",
        &[
            "sql/schema.sql",
            "sql/stored.sql",
            "sql/join.sql",
            "sql/note.sql",
            "sql/ambiguous.sql",
            "sql/unique.sql",
            "sql/virtual.sql",
        ],
    )
    .join("\n");
    assert!(join_only.contains("JOIN ON"), "{join_only}");
    assert!(!join_only.contains("WHERE filters"), "{join_only}");
}

#[test]
fn stale_extra_generated_columns_are_reported() {
    let body = messages(
        "fail",
        "sqlInclude: [\"sql/**/*.sql\"]\nextraGeneratedColumns:\n  - {table: orders, column: created_at, sourceColumn: id}\n",
        &[
            "sql/schema.sql",
            "sql/where.sql",
            "sql/order.sql",
            "sql/between.sql",
        ],
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
    let fail = [
        "sql/schema.sql",
        "sql/where.sql",
        "sql/order.sql",
        "sql/between.sql",
    ];
    assert_eq!(messages("fail", SQL, &fail), messages("fail", SQL, &fail));
}

#[test]
fn an_extra_column_absent_from_the_schema_is_tracked() {
    let body = messages(
        "pass",
        "sqlInclude: [\"sql/**/*.sql\"]\nextraGeneratedColumns:\n  - {table: invoices, column: created_at, sourceColumn: id}\n",
        &["sql/schema.sql", "sql/invoices.sql"],
    )
    .join("\n");
    assert!(body.contains("ORDER BY invoices.created_at"), "{body}");
}

#[test]
fn an_explicit_include_skips_other_files() {
    let body = messages(
        "fail",
        "include: [\"sql/schema.sql\", \"sql/where.sql\"]\nsqlInclude: [\"sql/**/*.sql\"]\n",
        &["sql/schema.sql", "sql/where.sql", "sql/order.sql"],
    )
    .join("\n");
    assert!(body.contains("WHERE filters"), "{body}");
    assert!(!body.contains("ORDER BY"), "{body}");
}

#[test]
fn no_tracked_columns_are_quiet_even_with_dynamic_sql() {
    let root = fixture("coverage");
    let file = root.join("src/query.ts");
    assert!(
        check_with_files(&root, &config_yaml("include: ['src/**/*.ts']\n"), &[file])
            .unwrap()
            .is_empty()
    );
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
