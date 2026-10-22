use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use std::path::PathBuf;

#[test]
fn graph_fact_map_uses_its_global_projection_for_per_file_coverage() {
    let root = normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/finite-set-consistency/call-literals/supplemental-skipped"),
    );
    let file = root.join("generated/schedules.test.mts");
    let calls = TsFactPlan {
        call_sites: true,
        ..Default::default()
    };
    let facts = collect_ts_facts(std::slice::from_ref(&file), calls);
    let lookup: &dyn TsFactLookup = &facts;
    assert!(lookup
        .get_ts_facts(&file)
        .is_some_and(|entry| !entry.call_sites.is_empty()));
    assert!(lookup.covers_ts_fact_plan_for_file(&file, calls));
    assert!(!lookup.covers_ts_fact_plan_for_file(&file, TsFactPlan::imports()));
}
