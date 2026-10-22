use super::CheckFactPlan;
use crate::codebase::postgres::EmbeddedSqlOptions;

#[test]
fn finite_call_paths_union_without_changing_global_graph_demand() {
    use std::{path::PathBuf, sync::Arc};
    let first = PathBuf::from("src/first.ts");
    let second = PathBuf::from("src/second.ts");
    let mut plan = CheckFactPlan {
        call_site_paths: Arc::new(vec![first.clone()]),
        ..Default::default()
    };
    let mut empty = CheckFactPlan::default();
    empty.include(plan.clone());
    assert!(Arc::ptr_eq(&empty.call_site_paths, &plan.call_site_paths));
    let shared = Arc::clone(&plan.call_site_paths);
    plan.include(CheckFactPlan {
        call_site_paths: Arc::clone(&shared),
        ..Default::default()
    });
    assert!(Arc::ptr_eq(&shared, &plan.call_site_paths));
    plan.include(CheckFactPlan {
        call_site_paths: Arc::new(vec![second.clone(), first.clone()]),
        ..Default::default()
    });
    assert_eq!(plan.call_site_paths.as_ref(), &[first, second]);
    assert!(!plan.graph.call_sites);
}

#[test]
fn production_path_demand_is_shared_across_plan_clones_and_combined_once() {
    use std::sync::Arc;
    let paths = Arc::new(
        [std::path::PathBuf::from("/fixture/index.ts")]
            .into_iter()
            .collect(),
    );
    let mut plan = CheckFactPlan {
        production_import_paths: Arc::clone(&paths),
        ..Default::default()
    };
    let cloned = plan.clone();
    assert!(Arc::ptr_eq(
        &plan.production_import_paths,
        &cloned.production_import_paths
    ));
    plan.include(cloned);
    plan.include(CheckFactPlan::default());
    assert!(Arc::ptr_eq(&plan.production_import_paths, &paths));
    plan.include(CheckFactPlan {
        production_import_paths: Arc::new(
            [std::path::PathBuf::from("/fixture/helper.ts")]
                .into_iter()
                .collect(),
        ),
        ..Default::default()
    });
    assert_eq!(plan.production_import_paths.len(), 2);
    assert_eq!(
        paths.len(),
        1,
        "union must not mutate another request's demand"
    );
    let mut empty = CheckFactPlan::default();
    empty.include(plan.clone());
    assert!(Arc::ptr_eq(
        &empty.production_import_paths,
        &plan.production_import_paths
    ));
}

#[test]
fn explicit_annotation_profiles_union_and_deduplicate() {
    let read = EmbeddedSqlOptions::configured("@example/db", &["read".into()]);
    let write = EmbeddedSqlOptions::configured("@example/db", &["write".into()]);
    let explicit = |profile| CheckFactPlan {
        query_annotation: true,
        query_annotation_options: Some(vec![profile]),
        ..Default::default()
    };

    let mut plan = explicit(write.clone());
    plan.include(CheckFactPlan::default());
    assert_eq!(plan.query_annotation_options, Some(vec![write.clone()]));
    plan.include(explicit(read.clone()));
    plan.include(explicit(write.clone()));
    assert_eq!(plan.query_annotation_options, Some(vec![read, write]));
}

#[test]
fn embedded_sql_rule_ids_union_without_duplicating_a_rule() {
    use crate::codebase::postgres::EmbeddedSqlRuleScan;
    let read = EmbeddedSqlOptions::configured("@example/db", &["read".into()]);
    let predicates = EmbeddedSqlRuleScan::new("postgres-required-predicates", &[], &[]).unwrap();
    let columns = EmbeddedSqlRuleScan::new("postgres-explicit-columns", &[], &[]).unwrap();
    let excluded = EmbeddedSqlRuleScan::new(
        "postgres-required-predicates",
        &[],
        &["src/legacy/**".into()],
    )
    .unwrap();
    let mut plan = CheckFactPlan {
        embedded_sql_rule_ids: [(read.clone(), vec![predicates.clone()])].into(),
        ..Default::default()
    };
    plan.include(CheckFactPlan {
        embedded_sql_rule_ids: [(read.clone(), vec![predicates, columns, excluded])].into(),
        ..Default::default()
    });
    let ids: Vec<_> = plan
        .embedded_sql_rule_ids
        .get(&read)
        .unwrap()
        .iter()
        .map(|scan| scan.rule_id.as_str())
        .collect();
    assert_eq!(
        ids,
        [
            "postgres-required-predicates",
            "postgres-explicit-columns",
            "postgres-required-predicates",
        ]
    );
}

#[test]
fn inactive_plan_does_not_widen_explicit_annotation_demand() {
    let read = EmbeddedSqlOptions::configured("@example/db", &["read".into()]);
    let write = EmbeddedSqlOptions::configured("@example/db", &["write".into()]);
    let mut plan = CheckFactPlan::default();
    plan.include(CheckFactPlan {
        query_annotation: true,
        query_annotation_options: Some(vec![read.clone()]),
        ..Default::default()
    });
    assert!(plan.query_annotation);
    assert_eq!(plan.query_annotation_options, Some(vec![read.clone()]));

    // An inactive request's options are not a demand and cannot add a profile.
    plan.include(CheckFactPlan {
        query_annotation_options: Some(vec![write]),
        ..Default::default()
    });
    assert_eq!(plan.query_annotation_options, Some(vec![read]));
}

#[test]
fn legacy_all_profile_annotation_demand_dominates_explicit_subsets() {
    let read = EmbeddedSqlOptions::configured("@example/db", &["read".into()]);
    let legacy = || CheckFactPlan {
        query_annotation: true,
        ..Default::default()
    };
    let explicit = || CheckFactPlan {
        query_annotation: true,
        query_annotation_options: Some(vec![read.clone()]),
        ..Default::default()
    };

    let mut legacy_first = legacy();
    legacy_first.include(explicit());
    assert!(legacy_first.query_annotation_options.is_none());

    let mut explicit_first = explicit();
    explicit_first.include(legacy());
    assert!(explicit_first.query_annotation_options.is_none());
}

#[test]
fn empty_explicit_subset_stays_empty_until_another_profile_is_requested() {
    let read = EmbeddedSqlOptions::configured("@example/db", &["read".into()]);
    let mut plan = CheckFactPlan {
        query_annotation: true,
        query_annotation_options: Some(Vec::new()),
        ..Default::default()
    };
    plan.include(CheckFactPlan::default());
    assert_eq!(plan.query_annotation_options, Some(Vec::new()));
    plan.include(CheckFactPlan {
        query_annotation: true,
        query_annotation_options: Some(vec![read.clone()]),
        ..Default::default()
    });
    assert_eq!(plan.query_annotation_options, Some(vec![read]));
}

#[test]
fn annotation_heads_survive_plan_union_as_import_and_binding_demand_only() {
    let profile = EmbeddedSqlOptions::configured("@bench/db", &["read".into()]);
    let mut plan = CheckFactPlan::default();
    plan.include(CheckFactPlan {
        query_annotation_heads: true,
        ..Default::default()
    });
    plan.include(CheckFactPlan {
        embedded_sql_options: vec![profile],
        ..Default::default()
    });

    let collected = plan.collected_ts_plan();
    assert!(plan.query_annotation_heads);
    assert!(!plan.query_annotation);
    assert!(collected.imports);
    assert!(collected.module_bindings);
    assert!(!collected.call_arguments);
}
