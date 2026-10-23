use super::*;

#[test]
fn call_site_demand_unions_without_copying_unchanged_shared_paths() {
    let mut plan = CheckFactPlan::default();
    plan.require_call_site_paths(&[PathBuf::from("/fixture/src/../entry.ts")]);
    let shared = Arc::clone(&plan.call_site_paths);
    plan.require_call_site_paths(&[]);
    plan.require_call_site_paths(&[PathBuf::from("/fixture/entry.ts")]);
    assert!(Arc::ptr_eq(&shared, &plan.call_site_paths));
    plan.include(plan.clone());
    plan.include(CheckFactPlan::default());
    assert!(Arc::ptr_eq(&shared, &plan.call_site_paths));
    assert!(plan
        .call_site_paths
        .contains(&PathBuf::from("/fixture/entry.ts")));
    let mut empty = CheckFactPlan::default();
    empty.include(plan.clone());
    assert!(Arc::ptr_eq(&shared, &empty.call_site_paths));
    let mut other = CheckFactPlan::default();
    other.require_call_site_paths(&[PathBuf::from("/fixture/helper.ts")]);
    plan.include(other);
    assert_eq!(plan.call_site_paths.len(), 2);
    assert_eq!(shared.len(), 1, "union must not widen another request");
    assert!(
        !plan.graph.call_sites,
        "scoped demand never widens broad demand"
    );
}
