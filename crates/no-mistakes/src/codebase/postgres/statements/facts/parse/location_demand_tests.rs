use super::*;
use crate::codebase::postgres::statements::facts::locations::work;
use crate::codebase::postgres::statements::SqlFactSite;

fn fixture(name: &str) -> String {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/postgres-facts");
    std::fs::read_to_string(root.join(name)).unwrap()
}

#[test]
fn conflict_projection_preserves_all_other_facts_and_reliability() {
    for name in [
        "source/conflict-location-demand.sql",
        "source/insert-conflict-expressions.sql",
        "source/insert-conflict-expressions-invalid.sql",
        "source/query-cte-select-conflict.sql",
        "source/query-cte-conflict-rejected.sql",
        "source/wrapper-recovery.sql",
        "embedded/variants-provenance-recovery.sql",
    ] {
        let sql = fixture(name);
        let mut full = extract_sql_variant_statement_facts(&sql, true, &[]);
        let narrow = extract_sql_statement_facts_with_location_demand(
            &sql,
            true,
            &[],
            SqlLocationDemand::Conflicts,
        );
        let sites = full.variant_locations.as_mut().unwrap();
        sites.locking.clear();
        sites.bound_tables.clear();
        sites
            .positions
            .retain(|site, _| matches!(site, SqlFactSite::Origin | SqlFactSite::Conflict(_)));
        assert_eq!(narrow, full, "{name}");
    }
}

#[test]
fn conflict_projection_omits_full_ast_lock_and_site_work() {
    let sql = fixture("source/conflict-location-demand.sql");
    work::begin();
    let narrow = extract_sql_statement_facts_with_location_demand(
        &sql,
        false,
        &[],
        SqlLocationDemand::Conflicts,
    );
    let narrow_work = work::finish();
    work::begin();
    let full = extract_sql_variant_statement_facts(&sql, false, &[]);
    let full_work = work::finish();
    work::begin();
    let legacy = extract_sql_statement_facts_with_recovered_placeholders(&sql, false, &[]);
    let legacy_work = work::finish();
    assert_eq!(narrow_work.full_walks, 0);
    assert_eq!(narrow_work.full_projections, 0);
    assert_eq!(narrow_work.conflict_projections, 1);
    assert!(full_work.full_walks >= 2);
    assert_eq!(full_work.full_projections, 1);
    assert_eq!(full_work.conflict_projections, 0);
    assert_eq!(legacy_work.full_walks, 0);
    assert_eq!(legacy_work.full_projections, 0);
    assert_eq!(legacy_work.conflict_projections, 0);
    assert!(narrow.variant_locations.unwrap().locking.is_empty());
    assert!(!full.variant_locations.unwrap().locking.is_empty());
    assert!(legacy.variant_locations.is_none());
}
