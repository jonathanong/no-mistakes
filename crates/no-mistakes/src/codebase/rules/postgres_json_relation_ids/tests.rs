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
fn insert_and_update_report_the_exact_missing_fk_guidance() {
    let findings = run(IDS, &["db/fail.sql"]);
    assert_eq!(findings.len(), 2, "{findings:#?}");
    for finding in findings {
        assert_eq!(finding.rule, RULE_ID);
        assert_eq!(finding.file, "db/fail.sql");
        assert_eq!(finding.target.as_deref(), Some("column:orders.details"));
        assert_eq!(finding.message, format!(
            "db/fail.sql:{}: key \"accountId\" written into orders.details has no same-row foreign key column; write account_id with a foreign key, or allow the column with a reason",
            finding.line
        ));
    }
}

#[test]
fn writing_the_mapped_fk_passes_for_each_row_and_each_statement() {
    assert!(run(IDS, &["db/pass.sql", "db/migration.sql"]).is_empty());
}

#[test]
fn other_table_foreign_keys_and_unwritten_mapped_columns_do_not_pass() {
    let findings = run(IDS, &["db/wrong-fk.sql"]);
    assert_eq!(findings.len(), 3, "{findings:#?}");
    assert_eq!(
        findings
            .iter()
            .filter(|f| f.target.as_deref() == Some("column:loose_orders.details"))
            .count(),
        2
    );
}

#[test]
fn matching_unmapped_keys_are_reported_without_inventing_a_column() {
    let findings = run(IDS, &["db/unmapped.sql"]);
    assert_eq!(findings.len(), 3, "{findings:#?}");
    for key in ["orderId", "id", "parent_id"] {
        assert!(
            findings
                .iter()
                .any(|f| f.message.contains(&format!("key \"{key}\""))),
            "{findings:#?}"
        );
    }
    assert!(findings.iter().all(|f| f
        .message
        .ends_with("write a mapped column with a foreign key, or allow the column with a reason")));
}

#[test]
fn nested_dynamic_and_non_jsonb_values_are_outside_the_check() {
    assert!(run(IDS, &["db/ignored.sql"]).is_empty());
}

#[test]
fn cte_dml_schema_qualified_tables_and_multiple_rows_are_checked() {
    let findings = run(IDS, &["db/edges.sql"]);
    assert_eq!(findings.len(), 4, "{findings:#?}");
    assert!(findings
        .iter()
        .all(|f| f.target.as_deref() == Some("column:orders.details")));
}

#[test]
fn allow_entries_apply_only_to_the_named_table_or_column() {
    assert_eq!(run(IDS, &["db/allow.sql"]).len(), 3);
    let table = format!("{IDS}allow: [{{object: 'table:invoice_revisions', reason: history}}]\n");
    let findings = run(&table, &["db/allow.sql"]);
    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert!(findings
        .iter()
        .all(|f| f.file == "db/allow.sql" && f.message.contains("orders.")));
    let column = format!("{IDS}allow: [{{object: 'column:orders.details', reason: snapshot}}]\n");
    let findings = run(&column, &["db/allow.sql"]);
    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert!(findings
        .iter()
        .any(|f| f.target.as_deref() == Some("column:orders.extra")));
    assert!(findings
        .iter()
        .any(|f| f.target.as_deref() == Some("column:invoice_revisions.changes")));
}

#[test]
fn fingerprint_columns_report_once_per_unique_key_and_use_exact_guidance() {
    let findings = run(
        "schemaCatalogPath: schema.json\nfingerprintNamePattern: '_sha256$'",
        &["schema.json"],
    );
    assert_eq!(findings.len(), 3, "{findings:#?}");
    for (column, key) in [
        ("fingerprint_sha256", "invoices_fingerprint_key"),
        ("fingerprint_sha256", "invoices_account_fingerprint_key"),
        ("document_sha256", "invoices_document_key"),
    ] {
        let finding = findings
            .iter()
            .find(|f| f.message.contains(&format!("UNIQUE key {key};")))
            .unwrap();
        assert_eq!(finding.file, "schema.json");
        assert_eq!(finding.target, Some(format!("column:invoices.{column}")));
        assert_eq!(finding.message, format!(
            "schema.json:{}: column invoices.{column} matches fingerprintNamePattern and is part of UNIQUE key {key}; confirm a relational key exists, or allow the column with a reason",
            finding.line
        ));
    }
}

#[test]
fn fingerprint_allow_entries_are_scoped_to_the_named_column_or_table() {
    let base = "schemaCatalogPath: schema.json\nfingerprintNamePattern: '_sha256$'\n";
    let column =
        format!("{base}allow: [{{object: 'column:invoices.fingerprint_sha256', reason: dedup}}]");
    assert_eq!(run(&column, &["schema.json"]).len(), 1);
    let table = format!("{base}allow: [{{object: 'table:invoices', reason: dedup}}]");
    assert!(run(&table, &["schema.json"]).is_empty());
}

#[test]
fn no_options_and_unset_patterns_report_nothing() {
    for yaml in [
        "{}",
        "schemaCatalogPath: schema.json",
        "schemaCatalogPath: schema.json\nkeyColumnMap: {accountId: account_id}",
    ] {
        assert!(run(yaml, &["db/fail.sql", "src/query.ts", "schema.json"]).is_empty());
    }
}

#[test]
fn sql_inputs_are_explicit_and_findings_are_deterministic() {
    let without_sql = IDS.replace(
        "sqlInclude: ['db/**/*.sql', 'suppress/**/*.sql']",
        "sqlInclude: ['not-selected/**/*.sql']",
    );
    assert!(run(&without_sql, &["db/fail.sql"]).is_empty());
    let files = ["db/unmapped.sql", "db/fail.sql", "db/edges.sql"];
    let first = run(IDS, &files);
    let second = run(IDS, &[files[2], files[1], files[0]]);
    assert_eq!(
        serde_json::to_vec(&first).unwrap(),
        serde_json::to_vec(&second).unwrap()
    );
}

#[test]
fn configured_json_checks_scan_sql_without_requiring_extra_globs() {
    let defaults = IDS.replace("sqlInclude: ['db/**/*.sql', 'suppress/**/*.sql']", "");
    assert_eq!(run(&defaults, &["db/fail.sql"]).len(), 2);
}

#[test]
fn file_filters_remain_explicit() {
    let excluded = format!("{IDS}exclude: ['db/fail.sql', 'src/query.ts']");
    assert!(run(&excluded, &["db/fail.sql", "src/query.ts"]).is_empty());
    let included = format!("{IDS}include: ['src/missing.ts']");
    assert!(run(&included, &["src/query.ts"]).is_empty());
}

#[test]
fn literal_syntax_and_query_arms_keep_same_row_fk_semantics() {
    assert!(run(IDS, &["db/syntax-pass.sql"]).is_empty());
    let findings = run(IDS, &["db/syntax-fail.sql"]);
    assert_eq!(findings.len(), 9, "{findings:#?}");
}

#[test]
fn positional_json_keys_without_a_catalog_remain_findings() {
    let findings = run(
        "idKeyPattern: Id$\nsqlInclude: ['db/*.sql']",
        &["db/positional.sql"],
    );
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].target.as_deref(), Some("table:events"));
    assert!(findings[0]
        .message
        .contains("key \"accountId\" written into positional column 1 of events"));
}
