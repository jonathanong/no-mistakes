use super::*;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::postgres::SchemaCatalog;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;
use std::sync::Arc;

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-required-predicates/fixture")
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

const COLUMNS: &str = r#"
sqlInclude: ["sql/**/*.sql"]
schemaCatalogPath: schema.json
partitionKeys: require
relations:
  - table: orders
    requireColumns: [account_id]
"#;

fn run(dir: &str, yaml: &str, file: &str) -> Vec<RuleFinding> {
    let root = fixture(dir);
    let sql = root.join(file);
    check_with_files(&root, &config_yaml(yaml), std::slice::from_ref(&sql)).unwrap()
}

fn messages(findings: &[RuleFinding]) -> String {
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
    ] {
        let findings = run("columns", COLUMNS, file);
        assert!(findings.is_empty(), "{file} {findings:?}");
    }
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

#[test]
fn expression_keys_are_catalog_findings_and_exemptions_skip_them() {
    let findings = run(
        "expression",
        "sqlInclude: [\"sql/**/*.sql\"]\nschemaCatalogPath: schema.json\npartitionKeys: require\n",
        "sql/001.sql",
    );
    assert!(
        findings.iter().any(|finding| {
            finding.file == "schema.json"
                && finding.line == 1
                && finding.target.as_deref() == Some("table:events")
                && finding.message.contains(
                    "cannot derive a column from partition key expression date_trunc('day'::text, created_at)"
                )
        }),
        "{findings:?}"
    );
    let exempt = run(
        "expression",
        "sqlInclude: [\"sql/**/*.sql\"]\nschemaCatalogPath: schema.json\npartitionKeys: require\n\
partitionKeyExemptions:\n  - {table: events, reason: expression key}\n",
        "sql/001.sql",
    );
    assert!(exempt.is_empty(), "{exempt:?}");
}

#[test]
fn stale_exemption_is_reported_at_the_catalog_path() {
    let findings = run(
        "stale-exemption",
        "sqlInclude: [\"sql/**/*.sql\"]\nschemaCatalogPath: schema.json\npartitionKeys: require\n\
partitionKeyExemptions:\n  - {table: topics, reason: not partitioned}\n",
        "sql/001.sql",
    );
    assert!(findings.iter().any(|finding| {
        finding.file == "schema.json"
            && finding.line == 1
            && finding
                .message
                .contains("stale postgres-required-predicates partitionKeyExemptions entry: topics")
    }));
}

#[test]
fn allow_suppresses_expression_findings_and_stale_entries_are_reported() {
    let allowed = run(
        "allow",
        "sqlInclude: [\"sql/**/*.sql\"]\nschemaCatalogPath: schema.json\npartitionKeys: require\n\
allow:\n  - {object: table:events, reason: reviewed}\n",
        "sql/001.sql",
    );
    assert!(
        allowed
            .iter()
            .all(|finding| !finding.message.contains("cannot derive")),
        "{allowed:?}"
    );
    assert!(allowed.iter().all(|finding| !finding
        .message
        .contains("stale postgres-required-predicates allow")));
    let stale = run(
        "allow",
        "sqlInclude: [\"sql/**/*.sql\"]\nschemaCatalogPath: schema.json\npartitionKeys: require\n\
allow:\n  - {object: table:missing, reason: reviewed}\n",
        "sql/001.sql",
    );
    assert!(stale.iter().any(|finding| {
        finding
            .message
            .contains("stale postgres-required-predicates allow entry: table:missing")
    }));
    assert!(stale
        .iter()
        .any(|finding| finding.message.contains("cannot derive")));
}

#[test]
fn prepared_catalog_is_used_instead_of_loading_a_missing_file() {
    let root = fixture("columns");
    let sql = root.join("sql/fail-kind.sql");
    let schema = root.join("schema.json");
    let sources = super::super::source_store_for_files(&[schema.clone(), sql.clone()]);
    let catalog = SchemaCatalog::load(&root, "schema.json", &sources).unwrap();
    let mut facts = CheckFactMap::default();
    facts
        .postgres_schema_catalogs
        .insert("missing/schema.json".to_string(), Ok(Arc::new(catalog)));
    let findings = check_with_files_sources_and_facts(
        &root,
        &config_yaml(
            "sqlInclude: [\"sql/**/*.sql\"]\nschemaCatalogPath: missing/schema.json\npartitionKeys: require\n",
        ),
        std::slice::from_ref(&sql),
        &sources,
        &facts,
    )
    .unwrap();
    assert!(
        messages(&findings).contains("partition key column account_id"),
        "{findings:?}"
    );
    let missing = check_with_files_sources_and_facts(
        &root,
        &config_yaml(
            "sqlInclude: [\"sql/**/*.sql\"]\nschemaCatalogPath: missing/schema.json\npartitionKeys: require\n",
        ),
        std::slice::from_ref(&sql),
        &sources,
        &CheckFactMap::default(),
    );
    assert!(
        missing.is_err(),
        "load path must fail when the catalog file is absent"
    );
}

#[test]
fn repeated_scans_are_identical() {
    let root = fixture("columns");
    let sql = root.join("sql/fail-ambiguous.sql");
    let config = config_yaml(COLUMNS);
    let first = check_with_files(&root, &config, std::slice::from_ref(&sql)).unwrap();
    let second = check_with_files(&root, &config, std::slice::from_ref(&sql)).unwrap();
    assert_eq!(first, second);
}

#[test]
fn suppression_directives_hide_sql_findings() {
    let root = fixture("suppress");
    let config = config_yaml(COLUMNS);
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
fn new_options_reject_invalid_values() {
    let cases = [
        (
            "partitionKeys: sideways\n",
            "option partitionKeys: expected require or off",
        ),
        (
            "partitionKeys: require\n",
            "option schemaCatalogPath: required when partitionKeys is require",
        ),
        (
            "relations:\n  - {table: orders, requireColumns: ['']}\n",
            "option requireColumns: empty column name",
        ),
        (
            "partitionKeys: require\nschemaCatalogPath: schema.json\npartitionKeyExemptions:\n  - {table: '', reason: reviewed}\n",
            "option partitionKeyExemptions: empty table name",
        ),
        (
            "partitionKeys: require\nschemaCatalogPath: schema.json\nallow:\n  - {object: '', reason: reviewed}\n",
            "option allow: empty object",
        ),
        (
            "partitionKeys: require\nschemaCatalogPath: schema.json\npartitionKeyExemptions:\n  - {table: events, reason: ''}\n",
            "option partitionKeyExemptions: empty reason",
        ),
        (
            "partitionKeys: require\nschemaCatalogPath: schema.json\npartitionKeyExemptions:\n  - {table: events, reason: a}\n  - {table: Events, reason: b}\n",
            "option partitionKeyExemptions: duplicate entry",
        ),
        (
            "partitionKeys: require\nschemaCatalogPath: schema.json\nallow:\n  - {object: table:events, reason: ''}\n",
            "option allow: empty reason",
        ),
        (
            "partitionKeys: require\nschemaCatalogPath: schema.json\nallow:\n  - {object: table:events, reason: a}\n  - {object: table:events, reason: b}\n",
            "option allow: duplicate entry",
        ),
    ];
    for (yaml, needle) in cases {
        let Err(error) = compile_options(&serde_yaml::from_str::<Options>(yaml).unwrap()) else {
            panic!("{yaml} compiled");
        };
        assert!(error.to_string().contains(needle), "{yaml} {error}");
    }
    assert!(compile_options(&Options {
        relations: vec![RelationOption {
            table: "orders".into(),
            require_columns: vec!["account_id".into()],
            ..Default::default()
        }],
        ..Default::default()
    })
    .is_ok());
}

#[test]
fn textual_require_message_stays_on_the_same_select() {
    let root = fixture("fail");
    let findings = check_with_files(
        &root,
        &config_yaml(
            "sqlInclude: [\"sql/**/*.sql\"]\nrelations:\n  - table: topics\n    require: [\"parent_id IS NOT NULL\"]\n    requireColumns: [parent_id]\n",
        ),
        &[root.join("sql/001.sql")],
    )
    .unwrap();
    assert!(findings.iter().any(|finding| finding
        .message
        .contains("queries against topics must include `parent_id IS NOT NULL`")));
}
