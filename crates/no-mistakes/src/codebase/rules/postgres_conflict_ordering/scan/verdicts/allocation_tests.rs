use super::*;
use crate::codebase::postgres::prepare_rule_sql_facts;
use crate::codebase::rules::postgres_conflict_ordering::tests::{config, fixture};
use crate::codebase::rules::postgres_conflict_ordering::{compile_options, Options};
use std::sync::Arc;

#[test]
fn disabled_ledger_constructs_no_records_and_allocates_nothing() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("pass-canonical-order"));
    let path = root.join("src/insert.ts");
    let files = [path.clone(), root.join("schema.json")];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let facts =
        prepare_rule_sql_facts(&root, &files, Arc::clone(&sources), &config(), &[RULE_ID]).unwrap();
    let opts = compile_options(&Options {
        schema_catalog_path: "schema.json".into(),
        import_specifier: "@example/db".into(),
        ..Default::default()
    })
    .unwrap();
    let source = crate::codebase::rules::read_source(&sources, &path).unwrap();
    let embedded = facts.embedded_sql(&path, &opts.embedded).unwrap();
    let statements = crate::codebase::rules::index_sql_variants(
        facts
            .postgres_statements(&path, Some(&opts.embedded))
            .unwrap(),
    );
    let catalog = facts.postgres_ordering_catalog("schema.json").unwrap();
    let enabled = facts.with_fresh_sql_verdicts();
    // The recorder owns its opt-in boundary, even when a future caller omits
    // the outer scan guard. Count real allocations, not estimated work sites.
    let (_, disabled_allocations) = crate::test_support::allocations::observe(|| {
        record(
            "src/insert.ts",
            &source,
            embedded,
            &opts,
            catalog,
            &statements,
            &facts,
        );
        assert!(facts.for_sql_application(0, None).is_none());
    });
    assert_eq!(disabled_allocations, 0);
    assert!(facts.sql_verdicts().is_empty());
    let (_, enabled_allocations) = crate::test_support::allocations::observe(|| {
        record(
            "src/insert.ts",
            &source,
            embedded,
            &opts,
            catalog,
            &statements,
            &enabled,
        );
    });
    assert!(enabled_allocations > 0);
    assert_eq!(enabled.sql_verdicts().len(), embedded.calls.len());
    assert!(!embedded.calls.is_empty());
}
