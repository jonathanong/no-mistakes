use super::*;

fn config_project(config: &str, policy_name: &str, include: &str) -> ConfigProject {
    ConfigProject {
        config: Some(config.to_string()),
        policy_name: Some(policy_name.to_string()),
        runner_project_arg: Some(policy_name.to_string()),
        include: vec![include.to_string()],
        exclude: Vec::new(),
    }
}

#[test]
fn explicit_policy_replaces_each_matching_config_project() {
    let root = Path::new("");
    let policy = TestProjectPolicy {
        include: vec!["src/**/*.test.ts".to_string()],
        exclude: vec!["src/skip/**".to_string()],
        ..Default::default()
    };
    let mut projects = vec![
        config_project("vitest.node.ts", "shared", "node/**/*.test.ts"),
        config_project("vitest.browser.ts", "shared", "browser/**/*.test.ts"),
        config_project("vitest.other.ts", "other", "other/**/*.test.ts"),
    ];

    apply_explicit_policy_projects(
        root,
        None,
        &BTreeMap::from([("shared".to_string(), policy)]),
        &mut projects,
    );

    let shared = projects
        .iter()
        .filter(|project| project.policy_name.as_deref() == Some("shared"))
        .collect::<Vec<_>>();
    assert_eq!(shared.len(), 2);
    assert!(shared
        .iter()
        .any(|project| project.config.as_deref() == Some("vitest.node.ts")));
    assert!(shared
        .iter()
        .any(|project| project.config.as_deref() == Some("vitest.browser.ts")));
    assert_eq!(projects.len(), 3);
}
