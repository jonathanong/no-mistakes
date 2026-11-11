use super::*;

fn project(name: &str, concurrent: bool) -> ConfigProject {
    ConfigProject {
        config: None,
        workspace: false,
        policy_name: Some(name.to_string()),
        runner_project_arg: Some(name.to_string()),
        scope: None,
        include: vec!["**/*.test.mts".to_string()],
        exclude: Vec::new(),
        matcher_warnings: Vec::new(),
        vitest_setup: Vec::new(),
        vitest_concurrent: concurrent,
        declared_deadlines: Default::default(),
    }
}

#[test]
fn explicit_project_overrides_retain_concurrency_only_for_the_matching_name() {
    let merged = merge_explicit_projects(
        vec![project("parallel", true), project("sequential", false)],
        vec![project("parallel", false)],
    );
    assert_eq!(merged.len(), 2);
    assert!(
        merged
            .iter()
            .find(|p| p.policy_name.as_deref() == Some("parallel"))
            .unwrap()
            .vitest_concurrent
    );
    assert!(
        !merged
            .iter()
            .find(|p| p.policy_name.as_deref() == Some("sequential"))
            .unwrap()
            .vitest_concurrent
    );
}
