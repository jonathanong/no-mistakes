use super::construct_annotation_module_indexes;
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn construction_adapter_retains_prepared_syntax_and_counts_runtime_bindings() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-target-reuse");
    let facts = collect_ts_facts(
        &[root.join("entry.mts"), root.join("origin.mts")],
        TsFactPlan {
            imports: true,
            module_bindings: true,
            ..Default::default()
        },
    );
    let facts = facts
        .into_iter()
        .map(|(_, facts)| Arc::new(facts))
        .collect::<Vec<_>>();
    let indexes = construct_annotation_module_indexes(&facts);
    assert_eq!(indexes.binding_count(), 15);
    assert!(facts.iter().all(|facts| Arc::strong_count(facts) == 2));
    assert_eq!(construct_annotation_module_indexes(&[]).binding_count(), 0);
}
