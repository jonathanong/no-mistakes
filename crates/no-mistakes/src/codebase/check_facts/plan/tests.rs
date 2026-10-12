use super::CheckFactPlan;
use crate::codebase::postgres::EmbeddedSqlOptions;

#[test]
fn selected_jsx_demand_unions_without_mutating_shared_plans() {
    use std::{path::PathBuf, sync::Arc};
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/test-no-dependency-pins/prepared-jsx/src");
    let paths = Arc::new([root.join("pins.test.tsx")].into_iter().collect());
    let mut plan = CheckFactPlan {
        jsx_text_paths: Arc::clone(&paths),
        ..Default::default()
    };
    let cloned = plan.clone();
    plan.include(cloned);
    plan.include(CheckFactPlan::default());
    assert!(Arc::ptr_eq(&plan.jsx_text_paths, &paths));
    plan.include(CheckFactPlan {
        jsx_text_paths: Arc::new([root.join("pins.test.jsx")].into_iter().collect()),
        ..Default::default()
    });
    assert_eq!(plan.jsx_text_paths.len(), 2);
    assert_eq!(paths.len(), 1, "another request keeps its selected scope");
    let mut empty = CheckFactPlan::default();
    empty.include(plan.clone());
    assert_eq!(empty.jsx_text_paths, plan.jsx_text_paths);
}

#[test]
fn production_path_demand_is_shared_across_plan_clones_and_unioned_once() {
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
