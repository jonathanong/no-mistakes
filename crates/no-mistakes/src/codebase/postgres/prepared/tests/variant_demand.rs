use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_playwright_and_sources, CheckFactPlan,
};
use crate::codebase::postgres::EmbeddedSqlOptions;
use std::{path::PathBuf, sync::Arc};

#[test]
fn direct_rule_variant_demand_shares_one_parse_and_skips_legacy_calls() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/postgres-facts/embedded"),
    );
    let ts = root.join("variants-prepared-demand.ts");
    let files = vec![ts.clone()];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let profile = EmbeddedSqlOptions::configured("@example/db", &["query".into()]);
    let mut plan = CheckFactPlan::default();
    // Lock ordering, conflict ordering, and annotations union one demand.
    for _ in 0..3 {
        plan.include(CheckFactPlan {
            postgres_variants: true,
            embedded_sql: true,
            embedded_sql_options: vec![profile.clone()],
            ..Default::default()
        });
    }
    assert!(!plan.postgres_dml);
    crate::ast::begin_parse_count(&root);
    let prepared = collect_check_facts_with_graph_files_playwright_and_sources(
        &root,
        files,
        vec![],
        plan,
        None,
        Arc::clone(&sources),
    );
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(counts.get(&ts), Some(&1), "{counts:#?}");
    assert_eq!(sources.physical_read_count(), 1);
    let variants = prepared.postgres_statements(&ts, Some(&profile)).unwrap();
    assert_eq!(variants.len(), 6);
    assert!(variants.iter().all(|fact| fact
        .variant_locations
        .as_ref()
        .is_some_and(|locations| matches!(locations.call_index, 2..=4))));
    assert_eq!(
        variants
            .iter()
            .filter(|fact| !fact.variant_locations.as_ref().unwrap().locking.is_empty())
            .count(),
        2
    );
    assert_eq!(
        variants
            .iter()
            .filter(|fact| !fact
                .variant_locations
                .as_ref()
                .unwrap()
                .conflicts
                .is_empty())
            .count(),
        2
    );
    for _ in 0..3 {
        assert!(std::ptr::eq(
            variants,
            prepared.postgres_statements(&ts, Some(&profile)).unwrap()
        ));
    }
}
