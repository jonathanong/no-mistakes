use super::*;
use crate::codebase::check_facts::CheckFactPlan;

fn plan(name: &str) -> CheckFactPlan {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/postgres-facts/grouping/fixture");
    let source = std::fs::read_to_string(root.join(format!("{name}.yml"))).unwrap();
    let config = serde_yaml::from_str(&source).unwrap();
    let mut plan = CheckFactPlan::default();
    crate::codebase::postgres::profiles::configure_prepared_postgres_plan(
        &root, &config, &mut plan,
    )
    .unwrap();
    plan
}

#[test]
fn grouping_projection_requires_an_enabled_catalog_backed_rule() {
    for name in ["no-catalog", "empty-catalog", "disabled"] {
        let actual = plan(name);
        assert!(!actual.postgres_grouping, "{name}");
        assert!(!actual.wants_full_sql_locations(), "{name}");
    }
    let actual = plan("catalog");
    assert!(actual.postgres_grouping);
    assert!(actual.postgres_dml);
    assert!(actual.wants_full_sql_locations());
}

#[test]
fn grouping_demand_survives_plan_merging_in_both_directions() {
    let mut merged = CheckFactPlan::default();
    merged.include(plan("catalog"));
    assert!(merged.postgres_grouping);
    assert!(merged.wants_full_sql_locations());
    merged.include(plan("no-catalog"));
    assert!(merged.postgres_grouping);
    assert!(merged.wants_full_sql_locations());
}
