use super::CheckFactPlan;
use std::path::PathBuf;
use std::sync::Arc;

fn configured(paths: &[&str]) -> CheckFactPlan {
    CheckFactPlan {
        query_annotation: true,
        query_annotation_workspace_importers: Some(Arc::new(
            paths.iter().map(|path| PathBuf::from(*path)).collect(),
        )),
        ..Default::default()
    }
}

#[test]
fn configured_scopes_union_without_mutating_shared_input_or_unrelated_widening() {
    let mut plan = configured(&["/fixture/a.mts"]);
    let clone = plan.clone();
    let shared = Arc::clone(plan.query_annotation_workspace_importers.as_ref().unwrap());
    plan.include(clone);
    assert!(Arc::ptr_eq(
        plan.query_annotation_workspace_importers.as_ref().unwrap(),
        &shared
    ));
    plan.include(CheckFactPlan::default());
    plan.include(configured(&[]));
    assert!(Arc::ptr_eq(
        plan.query_annotation_workspace_importers.as_ref().unwrap(),
        &shared
    ));
    plan.include(configured(&["/fixture/b.mts", "/fixture/a.mts"]));
    assert_eq!(
        plan.query_annotation_workspace_importers
            .as_ref()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(shared.len(), 1);
    let mut initially_unrelated = CheckFactPlan::default();
    initially_unrelated.include(configured(&[]));
    assert!(initially_unrelated
        .query_annotation_workspace_importers
        .unwrap()
        .is_empty());
}

#[test]
fn only_annotation_requesting_unrestricted_plan_dominates_in_either_merge_order() {
    let direct = || CheckFactPlan {
        query_annotation: true,
        ..Default::default()
    };
    let mut plan = configured(&["/fixture/a.mts"]);
    plan.include(direct());
    assert!(plan.query_annotation_workspace_importers.is_none());
    let mut plan = direct();
    plan.include(configured(&["/fixture/a.mts"]));
    assert!(plan.query_annotation_workspace_importers.is_none());
}
