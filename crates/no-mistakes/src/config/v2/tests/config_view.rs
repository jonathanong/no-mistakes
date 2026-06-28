use super::*;

#[test]
fn config_view_projects_of_type_filter() {
    let cfg = load_v2_config(&fixture("multi-project"), None).unwrap();
    let view = ConfigView::new(&cfg);
    let nextjs = view.projects_of_type(Some(&ProjectType::Nextjs));
    assert_eq!(nextjs.len(), 1);
    assert_eq!(nextjs[0].0, "web");
    let all = view.projects_of_type(None);
    assert_eq!(all.len(), 2);
}

#[test]
fn config_view_nextjs_root() {
    let cfg = load_v2_config(&fixture("multi-project"), None).unwrap();
    let view = ConfigView::new(&cfg);
    assert_eq!(view.nextjs_root(), "web");
}

#[test]
fn config_view_nextjs_root_default() {
    let cfg = NoMistakesConfig::default();
    let view = ConfigView::new(&cfg);
    assert_eq!(view.nextjs_root(), "app");
}

#[test]
fn config_view_server_route_globs_are_project_root_relative() {
    let cfg = load_v2_config(&fixture("server-route-globs"), None).unwrap();
    let view = ConfigView::new(&cfg);
    assert_eq!(
        view.server_route_globs(),
        vec![
            "backend/api/**".to_string(),
            "backend/legacy/**".to_string()
        ]
    );
}

#[test]
fn config_view_server_route_globs_skips_projects_without_routes() {
    let mut cfg = NoMistakesConfig::default();
    cfg.projects.insert(
        "backend".to_string(),
        Project {
            type_: Some(ProjectType::Server),
            root: Some("backend".to_string()),
            ..Default::default()
        },
    );
    let view = ConfigView::new(&cfg);
    assert!(view.server_route_globs().is_empty());
}

#[test]
fn config_view_server_route_globs_skips_empty_projects_in_fixture() {
    let cfg = load_v2_config(&fixture("server-route-globs-empty-project"), None).unwrap();
    let view = ConfigView::new(&cfg);
    assert_eq!(view.server_route_globs(), vec!["api/routes/**"]);
}

#[test]
fn config_view_playwright_configs() {
    let cfg = load_v2_config(&fixture("multi-project"), None).unwrap();
    let view = ConfigView::new(&cfg);
    let configs = view.playwright_configs().unwrap();
    assert_eq!(configs, vec!["playwright.config.ts"]);
}

#[test]
fn config_view_playwright_configs_none() {
    let cfg = NoMistakesConfig::default();
    let view = ConfigView::new(&cfg);
    assert!(view.playwright_configs().is_none());
}

#[test]
fn config_view_vitest_and_jest_configs() {
    let yaml = r#"
tests:
  vitest:
    configs: vitest.config.mts
  jest:
    configs:
      - jest.config.mjs
"#;
    let cfg: NoMistakesConfig = serde_yaml::from_str(yaml).unwrap();
    let view = ConfigView::new(&cfg);
    assert_eq!(view.vitest_configs().unwrap(), vec!["vitest.config.mts"]);
    assert_eq!(view.jest_configs().unwrap(), vec!["jest.config.mjs"]);
}

#[test]
fn config_view_selectors_and_filesystem() {
    let cfg = load_v2_config(&fixture("multi-project"), None).unwrap();
    let view = ConfigView::new(&cfg);
    assert_eq!(view.test_id_attributes(), &["data-testid", "data-pw"]);
    assert!(!view.html_ids());
    assert_eq!(view.component_selector_attributes()["dataPw"], "data-pw");
    assert_eq!(view.selector_roots(), &["web/app", "web/components"]);
    assert!(view
        .selector_exclude()
        .contains(&"**/*.test.tsx".to_string()));
    assert_eq!(view.skip_directories(), &[".next", "node_modules"]);
}

#[test]
fn config_view_project_rules() {
    let cfg = load_v2_config(&fixture("multi-project"), None).unwrap();
    let view = ConfigView::new(&cfg);
    assert!(view
        .project_rules("backend")
        .contains(&"http-route-static-paths"));
    assert!(view.project_rules("nonexistent").is_empty());
}

#[test]
fn config_view_unknown_project_targets_are_empty() {
    let cfg = load_v2_config(&fixture("unknown-rule-project-target"), None).unwrap();
    let view = ConfigView::new(&cfg);
    assert!(view.rule("unique-exports").is_none());
    assert!(view.project_rules("missing").is_empty());
    assert!(view.enabled_rules_for("missing").is_empty());
}

#[test]
fn config_view_rule_lookup() {
    let cfg = load_v2_config(&fixture("multi-project"), None).unwrap();
    let view = ConfigView::new(&cfg);
    assert!(view.rule("http-route-static-paths").is_some());
    assert!(view.rule("nonexistent-rule").is_none());
}

#[test]
fn rule_configured_requires_an_effective_target() {
    let unknown_project = load_v2_config(&fixture("unknown-rule-project-target"), None).unwrap();
    assert!(!unknown_project.rule_configured("unique-exports"));
    let repository = load_v2_config(&fixture("repository-and-project-rule"), None).unwrap();
    assert!(repository.rule_configured("unique-exports"));
    let test_target = load_v2_config(&fixture("rule-test-target"), None).unwrap();
    assert!(test_target.rule_configured("test-no-unmocked-dynamic-imports"));
    let yaml = r#"
rules:
  - rule: playwright-prefer-test-id-locators
    tests:
      playwright: [web]
"#;
    let cfg: NoMistakesConfig = serde_yaml::from_str(yaml).unwrap();
    assert!(cfg.rule_configured("playwright-prefer-test-id-locators"));
    let non_test_rule = load_v2_config(&fixture("non-test-rule-test-target"), None).unwrap();
    assert!(!non_test_rule.rule_configured("unique-exports"));
}

#[test]
fn playwright_rules_do_not_accept_vitest_test_targets() {
    let yaml = r#"
rules:
  - rule: playwright-unique-test-ids
    tests:
      vitest: [unit]
"#;
    let cfg: NoMistakesConfig = serde_yaml::from_str(yaml).unwrap();
    assert!(!cfg.rule_configured("playwright-unique-test-ids"));
}

#[test]
fn config_view_enabled_rules() {
    let cfg = load_v2_config(&fixture("multi-project"), None).unwrap();
    let view = ConfigView::new(&cfg);
    let rules = view.enabled_rules_for("backend");
    assert!(rules.iter().any(|(id, _)| *id == "http-route-static-paths"));
}

#[test]
fn config_view_disabled_rule_excluded() {
    let cfg = load_v2_config(&fixture("disabled-rule"), None).unwrap();
    let view = ConfigView::new(&cfg);
    let rules = view.enabled_rules_for("backend");
    assert!(rules.iter().any(|(id, _)| *id == "active-rule"));
    assert!(!rules.iter().any(|(id, _)| *id == "disabled-rule"));
}

#[test]
fn duplicate_stems_errors() {
    let err = load_v2_config(&fixture("duplicate-stems"), None)
        .err()
        .unwrap();
    assert!(err.to_string().contains("multiple config files found"));
}
