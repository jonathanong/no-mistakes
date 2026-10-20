use super::*;
use crate::codebase::postgres::collect::dml::embedded_call_facts_with_demand;
use crate::codebase::postgres::statements::facts::locations::work;
use crate::codebase::postgres::{
    extract_embedded_sql_from_source, EmbeddedSqlKind, EmbeddedSqlOptions,
};

#[test]
fn mapped_non_insert_conflict_requests_omit_work_and_preserve_requested_facts() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/conflict-only-mapped-demand-gate.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let embedded = extract_embedded_sql_from_source(
        &path,
        &source,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert_eq!(embedded.calls.len(), 11);
    for (index, call) in embedded.calls.iter().enumerate() {
        assert!(
            call.variants.is_empty() && !call.sql_source_positions.is_empty(),
            "call {index}: {call:?}"
        );
    }
    assert_eq!(embedded.calls[7].kind, EmbeddedSqlKind::Dynamic);
    assert_eq!(embedded.calls[10].kind, EmbeddedSqlKind::Dynamic);
    work::begin();
    let narrow =
        embedded_call_facts_with_demand(&embedded, false, false, SqlLocationDemand::Conflicts);
    assert_eq!(work::finish().statement_extractions, 3);
    assert_eq!(narrow.len(), 3);
    assert_eq!(
        narrow
            .iter()
            .map(|facts| facts.variant_locations.as_ref().unwrap().call_index)
            .collect::<Vec<_>>(),
        [4, 5, 6]
    );
    assert_eq!(
        narrow[0]
            .variant_locations
            .as_ref()
            .unwrap()
            .conflicts
            .len(),
        1
    );
    // Keep actual conflict sites and quoted/comment false positives unchanged.
    let complete =
        embedded_call_facts_with_demand(&embedded, false, false, SqlLocationDemand::Full);
    assert_eq!(narrow, complete[4..7]);
    for (demand, bounds) in [
        (SqlLocationDemand::None, false),
        (SqlLocationDemand::Full, false),
        (SqlLocationDemand::Conflicts, true),
    ] {
        work::begin();
        let facts = embedded_call_facts_with_demand(&embedded, bounds, false, demand);
        assert_eq!(work::finish().statement_extractions, 11);
        assert_eq!(facts.len(), 11);
        assert!(facts[1].parse_failed && facts[3].parse_failed);
    }
}
