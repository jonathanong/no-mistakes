use crate::codebase::postgres::statements::SqlLocationDemand;
use no_mistakes_postgres_sql::statements::work;

#[test]
fn conflict_only_demand_skips_irrelevant_statement_extractions() {
    use crate::codebase::postgres::collect::dml::embedded_call_facts_with_demand;
    use crate::codebase::postgres::{extract_embedded_sql_from_source, EmbeddedSqlOptions};
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/conflict-only-demand-gate.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let file = extract_embedded_sql_from_source(
        &path,
        &source,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert_eq!(file.calls.len(), 7);
    assert!(file
        .calls
        .iter()
        .all(|call| call.variants.is_empty() && call.sql_source_positions.is_empty()));
    work::begin();
    let narrow = embedded_call_facts_with_demand(&file, false, false, SqlLocationDemand::Conflicts);
    let narrow_work = work::finish();
    assert_eq!(narrow_work.statement_extractions, 2);
    assert_eq!(narrow.len(), 2);
    // Keep conservative quoted-text false positives and SELECT-rooted mutating CTEs.
    assert_eq!(narrow[0].variant_locations.as_ref().unwrap().call_index, 1);
    assert_eq!(narrow[1].variant_locations.as_ref().unwrap().call_index, 2);
    assert_eq!(
        narrow[1]
            .variant_locations
            .as_ref()
            .unwrap()
            .conflicts
            .len(),
        1
    );
    for demand in [SqlLocationDemand::None, SqlLocationDemand::Full] {
        work::begin();
        let full = embedded_call_facts_with_demand(&file, false, false, demand);
        assert_eq!(work::finish().statement_extractions, 7);
        assert_eq!(full.len(), 7);
        assert!(
            full[4].parse_failed,
            "the saved SELECT control must be malformed"
        );
    }
    work::begin();
    let bounds = embedded_call_facts_with_demand(&file, true, false, SqlLocationDemand::Conflicts);
    assert_eq!(work::finish().statement_extractions, 7);
    assert_eq!(bounds.len(), 7);
}
