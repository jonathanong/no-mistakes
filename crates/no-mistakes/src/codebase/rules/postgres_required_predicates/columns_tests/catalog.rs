// Output cases moved to cli_postgres_snapshot_required_predicates.
use super::super::*;
use super::{config_yaml, fixture, messages};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::postgres::SchemaCatalog;
use std::sync::Arc;

#[test]
fn prepared_catalog_is_used_instead_of_loading_a_missing_file() {
    let root = fixture("columns");
    let sql = root.join("sql/fail-kind.sql");
    let schema = root.join("schema.json");
    let sources = super::super::super::source_store_for_files(&[schema.clone(), sql.clone()]);
    let catalog = SchemaCatalog::load(&root, "schema.json", &sources).unwrap();
    let mut facts = crate::codebase::postgres::prepare_rule_sql_facts(&root, std::slice::from_ref(&sql), Arc::clone(&sources), &config_yaml("sqlInclude: [sql/**/*.sql]\nrelations: [{table: events, requireColumns: [account_id]}]\n"), &[RULE_ID]).unwrap();
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
    let mut failed = facts;
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
