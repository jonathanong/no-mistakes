use super::*;
use crate::codebase::postgres::statements::facts::locations::work;
use crate::codebase::postgres::statements::{SqlFactSite, SqlVariantLocations};

fn conflict_sites(mut locations: SqlVariantLocations) -> SqlVariantLocations {
    locations.locking.clear();
    locations.bound_tables.clear();
    locations.positions.retain(|site, _| {
        matches!(
            site,
            SqlFactSite::Origin | SqlFactSite::Conflict(_) | SqlFactSite::ParseFailure(_)
        )
    });
    locations
}

pub(super) fn policy_only(full: SqlStatementFileFacts) -> SqlStatementFileFacts {
    SqlStatementFileFacts {
        path: full.path,
        origin_line: full.origin_line,
        parse_failed: full.parse_failed,
        parse_failures: full.parse_failures,
        variant_locations: full.variant_locations.map(conflict_sites),
        ..Default::default()
    }
}

#[test]
fn conflict_policy_projection_omits_general_facts_and_preserves_recovery() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/source");
    for name in [
        "conflict-location-demand.sql",
        "insert-conflict-expressions.sql",
        "insert-conflict-expressions-invalid.sql",
        "query-cte-select-conflict.sql",
        "query-cte-conflict-rejected.sql",
        "wrapper-recovery.sql",
        "../embedded/variants-provenance-recovery.sql",
    ] {
        let sql = std::fs::read_to_string(root.join(name)).unwrap();
        work::begin();
        let full = extract_sql_variant_statement_facts(&sql, false, &[]);
        let full_work = work::finish();
        work::begin();
        let narrow = extract_sql_conflict_policy_facts(&sql, &[]);
        let narrow_work = work::finish();
        assert_eq!(full_work.general_fact_collections, 1, "{name}");
        assert_eq!(narrow_work.general_fact_collections, 0, "{name}");
        assert_eq!(narrow_work.statement_extractions, 1, "{name}");
        assert_eq!(narrow_work.full_walks, 0, "{name}");
        assert_eq!(narrow_work.conflict_projections, 1, "{name}");
        assert_eq!(narrow, policy_only(full.clone()), "{name}");
        // The old location-demand API still supplies all requested general facts.
        work::begin();
        let complete = extract_sql_statement_facts_with_location_demand(
            &sql,
            false,
            &[],
            SqlLocationDemand::Conflicts,
        );
        assert_eq!(work::finish().general_fact_collections, 1, "{name}");
        let mut expected = full;
        expected.variant_locations = expected.variant_locations.map(conflict_sites);
        assert_eq!(complete, expected, "{name}");
    }
}
