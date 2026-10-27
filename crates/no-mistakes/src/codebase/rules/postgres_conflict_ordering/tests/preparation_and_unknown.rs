#[test]
fn requires_a_catalog_path() {
    let error = match compile_options(&Options::default()) {
        Ok(_) => panic!("missing schemaCatalogPath should fail"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("schemaCatalogPath"));
}

#[test]
fn compile_options_canonicalize_executor_names_for_prepared_fact_lookup() {
    let compiled = compile_options(&Options {
        schema_catalog_path: "schema.json".to_string(),
        executor_names: vec![
            "write".to_string(),
            "query".to_string(),
            "write".to_string(),
        ],
        ..Default::default()
    })
    .unwrap();
    assert_eq!(compiled.embedded.executor_names, ["query", "write"]);
}

#[test]
fn prepared_scan_contextualizes_a_missing_embedded_sql_projection() {
    let root = fixture("pass-canonical-order");
    let file = root.join("src/insert.ts");
    let inventory = std::sync::Arc::new(crate::codebase::ts_source::FileInventory::from_paths(
        std::slice::from_ref(&file),
    ));
    let sources = crate::codebase::ts_source::SourceStore::new(inventory);
    let mut facts = crate::codebase::check_facts::CheckFactMap::default();
    facts.postgres_schema_catalogs.insert(
        "schema.json".to_string(),
        Ok(std::sync::Arc::new(
            crate::codebase::postgres::SchemaCatalog::default(),
        )),
    );
    let compiled = compile_options(&Options {
        schema_catalog_path: "schema.json".to_string(),
        ..Options::default()
    })
    .unwrap();

    let error = scan::scan_with_sources(
        &root,
        &compiled,
        std::slice::from_ref(&file),
        &sources,
        &facts,
    )
    .expect_err("missing prepared facts must fail closed");
    assert!(error.to_string().contains(
        "postgres-conflict-ordering failed to collect embedded SQL facts from prepared analysis"
    ));
    assert!(format!("{error:#}").contains(&file.display().to_string()));
}
