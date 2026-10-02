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
            .join("../../test-cases/rules/postgres-explicit-columns/fixture")
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

const SQL: &str =
    "sqlInclude: [\"sql/**/*.sql\"]\nschemaCatalogPath: schema.json\nrelations: [accounts]\n";

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
    let found = messages(
        "fail",
        SQL,
        &[
            "sql/star.sql",
            "sql/join.sql",
            "sql/returning.sql",
            "sql/cte.sql",
            "sql/accounts.sql",
        ],
    );
    let body = found.join("\n");
    assert!(
        body.contains("SELECT * reads all 40 columns of orders (limit 12)"),
        "{body}"
    );
    assert!(
        body.contains("RETURNING * returns all 40 columns of orders (limit 12)"),
        "{body}"
    );
    assert!(
        body.contains("SELECT * reads all columns of accounts, which is configured in relations"),
        "{body}"
    );
    assert_eq!(
        found
            .iter()
            .filter(|message| message.contains("40 columns of orders"))
            .count(),
        4
    );
}

#[test]
fn valid_examples_are_quiet() {
    let files = [
        "sql/columns.sql",
        "sql/tags.sql",
        "sql/exists.sql",
        "sql/count.sql",
        "sql/json.sql",
        "sql/cte.sql",
        "sql/missing.sql",
    ];
    assert!(messages("pass", SQL, &files).is_empty());
}

#[test]
fn max_columns_zero_reports_narrow_tables_and_skips_views() {
    let found = messages(
        "max-zero",
        "sqlInclude: [\"sql/**/*.sql\"]\nschemaCatalogPath: schema.json\nmaxColumns: 0\n",
        &["sql/tags.sql", "sql/view.sql"],
    );
    let body = found.join("\n");
    assert!(
        body.contains("SELECT * reads every column of tags, including columns added later"),
        "{body}"
    );
    assert!(!body.contains("view_public_tags"), "{body}");
}

#[test]
fn edges_cover_joins_qualification_and_returning_switch() {
    let wide = messages("edges", SQL, &["sql/join-wide.sql", "sql/qualified.sql"]);
    let body = wide.join("\n");
    assert!(body.contains("40 columns of orders"), "{body}");
    assert!(body.contains("20 columns of shipments"), "{body}");
    assert_eq!(
        wide.iter()
            .filter(|message| message.contains("orders"))
            .count(),
        2
    );
    assert!(messages("edges", SQL, &["sql/cte-alias.sql", "sql/json-upper.sql"]).is_empty());
    assert!(messages(
        "edges",
        "sqlInclude: [\"sql/**/*.sql\"]\nschemaCatalogPath: schema.json\ncheckReturning: false\n",
        &["sql/returning-only.sql"],
    )
    .is_empty());
    let listed = messages(
        "edges",
        "sqlInclude: [\"sql/**/*.sql\"]\nschemaCatalogPath: schema.json\nallowWholeRowFunctions: []\n",
        &["sql/json-upper.sql"],
    );
    assert!(
        listed.join("\n").contains("40 columns of orders"),
        "{listed:?}"
    );
}

#[test]
fn default_sql_include_leaves_sql_files_unscanned() {
    assert!(messages(
        "edges",
        "schemaCatalogPath: schema.json\n",
        &["sql/qualified.sql"]
    )
    .is_empty());
}

#[test]
fn repeated_scans_match() {
    let first = messages("fail", SQL, &["sql/star.sql"]);
    let second = messages("fail", SQL, &["sql/star.sql"]);
    assert_eq!(first, second);
}

#[test]
fn suppression_directives_hide_stars() {
    let root = fixture("suppress");
    let files = ["sql/next-line.sql", "sql/line.sql", "sql/file.sql"];
    let paths: Vec<PathBuf> = files.iter().map(|file| root.join(file)).collect();
    let mut findings = check_with_files(&root, &config_yaml(SQL), &paths).unwrap();
    assert_eq!(findings.len(), 3, "{findings:?}");
    let sources = super::super::source_store_for_files(&paths);
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn prepared_catalog_is_preferred_when_present() {
    let root = fixture("edges");
    let sql = root.join("sql/qualified.sql");
    let sources = super::super::source_store_for_files(&[root.join("schema.json"), sql.clone()]);
    let catalog = SchemaCatalog::load(&root, "schema.json", &sources).unwrap();
    let mut facts = CheckFactMap::default();
    facts
        .postgres_schema_catalogs
        .insert("missing/schema.json".to_string(), Ok(Arc::new(catalog)));
    let yaml = "sqlInclude: [\"sql/**/*.sql\"]\nschemaCatalogPath: missing/schema.json\n";
    let findings = check_with_files_sources_and_facts(
        &root,
        &config_yaml(yaml),
        std::slice::from_ref(&sql),
        &sources,
        &facts,
    )
    .unwrap();
    assert!(findings[0].message.contains("40 columns of public.orders"));
    let mut broken = CheckFactMap::default();
    broken.postgres_schema_catalogs.insert(
        "missing/schema.json".to_string(),
        Err(Arc::<str>::from("catalog broke")),
    );
    let error = check_with_files_sources_and_facts(
        &root,
        &config_yaml(yaml),
        std::slice::from_ref(&sql),
        &sources,
        &broken,
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("catalog broke"), "{error}");
    let fallback = check_with_files_sources_and_facts(
        &root,
        &config_yaml(SQL),
        std::slice::from_ref(&sql),
        &sources,
        &CheckFactMap::default(),
    )
    .unwrap();
    assert!(fallback[0].message.contains("40 columns of public.orders"));
}

#[test]
fn invalid_include_glob_is_a_config_error() {
    let root = fixture("edges");
    let error = check_with_files(
        &root,
        &config_yaml("schemaCatalogPath: schema.json\ninclude: ['[']\n"),
        &[],
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("invalid glob"), "{error}");
}
