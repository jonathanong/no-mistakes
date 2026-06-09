use super::*;

fn scoped_project(config: &str, scope: Option<&str>) -> ConfigProject {
    ConfigProject {
        config: Some(config.to_string()),
        policy_name: None,
        runner_project_arg: Some("chromium".to_string()),
        scope: scope.map(str::to_string),
        include: Vec::new(),
        exclude: Vec::new(),
    }
}

#[test]
fn strict_descendant_detects_nested_directories() {
    assert!(is_strict_descendant(
        "playwright",
        "playwright/credentialed"
    ));
    assert!(is_strict_descendant("", "playwright"));
    assert!(is_strict_descendant(".", "playwright"));
    assert!(!is_strict_descendant("playwright", "playwright"));
    assert!(!is_strict_descendant("playwright", "playwright-extra/foo"));
    assert!(!is_strict_descendant(
        "playwright/credentialed",
        "playwright"
    ));
    assert!(!is_strict_descendant("", ""));
}

#[test]
fn owning_projects_drops_ancestor_config_for_nested_spec() {
    let broad = scoped_project("playwright.config.mts", Some("playwright"));
    let nested = scoped_project(
        "playwright.credentialed.config.mts",
        Some("playwright/credentialed"),
    );
    let matched = vec![&broad, &nested];
    let owners = owning_projects(&matched);
    let configs: Vec<_> = owners
        .iter()
        .map(|project| project.config.as_deref().unwrap())
        .collect();
    assert_eq!(configs, vec!["playwright.credentialed.config.mts"]);
}

#[test]
fn owning_projects_keeps_sibling_and_equal_scopes() {
    let web = scoped_project("playwright.config.mts", Some("playwright/web"));
    let storybook = scoped_project("playwright.storybook.mts", Some("playwright/storybook"));
    let equal_a = scoped_project("vitest.node.mts", Some("src"));
    let equal_b = scoped_project("vitest.browser.mts", Some("src"));
    let matched = vec![&web, &storybook, &equal_a, &equal_b];
    assert_eq!(owning_projects(&matched).len(), 4);
}

#[test]
fn owning_projects_never_drops_policy_projects() {
    let broad = scoped_project("playwright.config.mts", Some("playwright"));
    let mut policy = scoped_project("playwright.credentialed.config.mts", None);
    policy.policy_name = Some("shared".to_string());
    let matched = vec![&broad, &policy];
    // The policy project (scope == None) survives and does not dominate the
    // broad config, so both targets remain.
    assert_eq!(owning_projects(&matched).len(), 2);
}
