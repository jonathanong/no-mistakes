use super::*;
use crate::codebase::postgres::prepare_rule_sql_facts;
use crate::codebase::rules::postgres_conflict_ordering::tests::{config, fixture};
use std::sync::Arc;

#[test]
fn missing_prepared_conflict_locations_return_contextual_errors() {
    let variant_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-conflict-ordering/variants/one-bad");
    for (root, name, variants) in [
        (fixture("pass-canonical-order"), "src/insert.ts", false),
        (variant_root, "src/query.ts", true),
    ] {
        let root = crate::codebase::ts_resolver::normalize_path(&root);
        let query = root.join(name);
        let files = [query.clone(), root.join("schema.json")];
        let sources = crate::codebase::rules::source_store_for_files(&files);
        let config = config();
        let mut facts =
            prepare_rule_sql_facts(&root, &files, Arc::clone(&sources), &config, &[RULE_ID])
                .unwrap();
        let compiled = super::super::compile_options(&super::super::Options {
            schema_catalog_path: "schema.json".into(),
            import_specifier: "@example/db".into(),
            ..Default::default()
        })
        .unwrap();
        assert_eq!(
            facts
                .embedded_sql(&query, &compiled.embedded)
                .unwrap()
                .calls
                .iter()
                .any(|call| !call.variants.is_empty()),
            variants
        );
        // Simulate an incomplete prepared projection without re-reading or parsing.
        for entry in facts.postgres.embedded.values_mut() {
            for statement in Arc::make_mut(entry.as_mut().unwrap()) {
                statement.variant_locations = None;
            }
        }
        let reads = sources.physical_read_count();
        let error = scan_with_sources(&root, &compiled, &[query], &sources, &facts)
            .expect_err("missing locations must fail closed instead of panicking");
        let message = format!("{error:#}");
        assert!(
            message.contains(if variants {
                "prepared SQL variant facts are missing"
            } else {
                "prepared SQL facts are missing"
            }),
            "{message}"
        );
        assert!(message.contains(name), "{message}");
        assert_eq!(sources.physical_read_count(), reads);
    }
}

#[test]
fn exact_missing_prepared_conflict_locations_return_an_error() {
    let sql = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres-facts/source/conflict-location-demand.sql"),
    )
    .unwrap();
    let facts = crate::codebase::postgres::extract_sql_statement_facts(&sql);
    assert!(facts.variant_locations.is_none());
    let error = exact::findings(
        "query.ts",
        7,
        &sql,
        &facts,
        &crate::codebase::postgres::SchemaCatalog::default(),
        true,
    )
    .expect_err("missing exact locations must fail closed");
    let message = format!("{error:#}");
    assert!(
        message.contains("prepared SQL conflict locations are missing"),
        "{message}"
    );
    assert!(message.contains("query.ts:7"), "{message}");
}
