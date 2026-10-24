use super::*;
use crate::codebase::postgres::sql_identifier_syntax::plan::SqlIdentifierScan;
use crate::config::v2::schema::EffectTargetConfig;

#[test]
fn typed_executor_demand_is_profiled_and_path_scoped() {
    let root = crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier")
        .join("effort-low");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let rule = config.rule_applications("postgres-no-unbounded-sql-identifier")[0];
    let typed = [EffectTargetConfig {
        module: "@example/postgres".into(),
        export: "QueryExecutor".into(),
        category: None,
    }];
    let selected = SqlIdentifierScan::new(&root, &config, rule, 0, &["src/query.ts".into()], &[])
        .unwrap()
        .with_typed_executor_types(&typed);
    let untyped =
        SqlIdentifierScan::new(&root, &config, rule, 1, &["src/selection.ts".into()], &[]).unwrap();
    let profile = EmbeddedSqlOptions::configured("@example/db", &[]);
    let other_profile = EmbeddedSqlOptions::configured("@unrelated/db", &[]);
    let unmatched_profile = EmbeddedSqlOptions::configured("@unused/db", &[]);
    let no_identity_profile = EmbeddedSqlOptions::configured("@no-types/db", &[]);
    let mut plan = CheckFactPlan::default();
    plan.sql_identifier_profiles
        .insert(profile.clone(), vec![selected.clone(), untyped]);
    plan.sql_identifier_profiles
        .insert(other_profile.clone(), vec![selected.clone()]);
    let unmatched =
        SqlIdentifierScan::new(&root, &config, rule, 3, &["src/excluded.ts".into()], &[])
            .unwrap()
            .with_typed_executor_types(&typed);
    plan.sql_identifier_profiles
        .insert(unmatched_profile, vec![unmatched]);
    plan.sql_identifier_profiles.insert(
        no_identity_profile,
        vec![
            SqlIdentifierScan::new(&root, &config, rule, 2, &["src/query.ts".into()], &[]).unwrap(),
        ],
    );

    let query = typed_executor_types_by_profile(&root, &root.join("src/query.ts"), &plan);
    assert_eq!(query.len(), 2);
    assert_eq!(query[0].0, profile);
    assert_eq!(query[1].0, other_profile);
    assert!(query.iter().all(|(_, types)| types == &typed));
    assert!(
        typed_executor_types_by_profile(&root, &root.join("src/selection.ts"), &plan).is_empty()
    );
}

#[test]
fn typed_executor_demand_sorts_and_deduplicates_module_export_identity() {
    let root = crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier")
        .join("effort-low");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let rule = config.rule_applications("postgres-no-unbounded-sql-identifier")[0];
    let duplicate = EffectTargetConfig {
        module: "@example/postgres".into(),
        export: "QueryExecutor".into(),
        category: None,
    };
    let another = EffectTargetConfig {
        module: "@example/pg".into(),
        export: "Executor".into(),
        category: None,
    };
    let scan = |index| {
        SqlIdentifierScan::new(&root, &config, rule, index, &["src/query.ts".into()], &[]).unwrap()
    };
    let profile = EmbeddedSqlOptions::configured("@example/db", &[]);
    let mut plan = CheckFactPlan::default();
    plan.sql_identifier_profiles.insert(
        profile.clone(),
        vec![
            scan(0).with_typed_executor_types(&[another.clone(), duplicate.clone()]),
            scan(1).with_typed_executor_types(&[duplicate.clone()]),
        ],
    );

    let demands = typed_executor_types_by_profile(&root, &root.join("src/query.ts"), &plan);
    assert_eq!(demands.len(), 1);
    assert_eq!(demands[0].0, profile);
    assert_eq!(
        demands[0].1,
        [another, duplicate].into_iter().collect::<Vec<_>>()
    );
}
