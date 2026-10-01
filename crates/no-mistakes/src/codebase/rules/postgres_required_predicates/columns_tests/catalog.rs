use super::super::*;
use super::{config_yaml, fixture, messages, run};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::postgres::SchemaCatalog;
use std::sync::Arc;

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
    let sources = super::super::super::source_store_for_files(&[schema.clone(), sql.clone()]);
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
    let mut failed = CheckFactMap::default();
    failed.postgres_schema_catalogs.insert(
        "missing/schema.json".to_string(),
        Err(Arc::<str>::from("catalog failed")),
    );
    let recorded = check_with_files_sources_and_facts(
        &root,
        &config_yaml(
            "sqlInclude: [\"sql/**/*.sql\"]\nschemaCatalogPath: missing/schema.json\npartitionKeys: require\n",
        ),
        std::slice::from_ref(&sql),
        &sources,
        &failed,
    );
    assert!(recorded.unwrap_err().to_string().contains("catalog failed"));
}
