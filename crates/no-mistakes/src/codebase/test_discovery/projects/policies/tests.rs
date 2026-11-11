use super::*;

#[test]
fn explicit_named_policy_keeps_runner_concurrency() {
    let policy = TestProjectPolicy {
        include: vec!["**/*.test.mts".to_string()],
        ..Default::default()
    };
    let prepared = configured_project(
        Path::new("."),
        "unit",
        &policy,
        None,
        false,
        RunnerDetails {
            concurrent: true,
            ..Default::default()
        },
    )
    .unwrap();
    let mut projects = vec![prepared];
    let policies = BTreeMap::from([("unit".to_string(), policy)]);
    apply_explicit_policy_projects(Path::new("."), None, &policies, &mut projects);
    assert_eq!(projects.len(), 1);
    assert!(projects[0].vitest_concurrent);
}
