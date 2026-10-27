use super::tests::{config_yaml, fixture};
use super::*;

const SQL: &str = "schemaCatalogPath: schema.json\nsqlInclude: [sql/**/*.sql]\n";

#[test]
fn overflowing_thresholds_are_configuration_errors() {
    let opts: Options =
        serde_yaml::from_str("schemaCatalogPath: schema.json\nmaxColumns: 4294967296\n").unwrap();
    assert!(compile_options(&opts)
        .err()
        .unwrap()
        .to_string()
        .contains("must be at most 4294967295"));
}

#[test]
fn prepared_scans_fail_on_absent_fact_demand_and_keep_sql_opt_in() {
    let root = fixture("deferred");
    let file = root.join("sql/lines.sql");
    let sources = super::super::source_store_for_files(std::slice::from_ref(&file));
    let opts = compile_options(&serde_yaml::from_str(SQL).unwrap()).unwrap();
    assert!(
        scan::scan(&root, &opts, std::slice::from_ref(&file), &sources, None)
            .unwrap_err()
            .to_string()
            .contains("prepared PostgreSQL facts are required")
    );
    let mut facts = crate::codebase::check_facts::CheckFactMap::default();
    let catalog =
        crate::codebase::postgres::SchemaCatalog::load(&root, "schema.json", &sources).unwrap();
    facts
        .postgres_schema_catalogs
        .insert("schema.json".into(), Ok(std::sync::Arc::new(catalog)));
    assert!(scan::scan(
        &root,
        &opts,
        std::slice::from_ref(&file),
        &sources,
        Some(&facts)
    )
    .unwrap_err()
    .to_string()
    .contains("prepared PostgreSQL facts are missing"));
    let config = config_yaml("schemaCatalogPath: schema.json\n");
    let prepared = crate::codebase::postgres::prepare_rule_sql_facts(
        &root,
        &[file],
        sources,
        &config,
        &[RULE_ID],
    )
    .unwrap();
    assert!(prepared.postgres.statements.is_empty());
}
