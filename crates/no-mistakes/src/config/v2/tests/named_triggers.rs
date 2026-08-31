use crate::config::v2::schema::{NamedFullSuiteTrigger, NoMistakesConfig};

#[test]
fn named_full_suite_triggers_list_form_does_not_need_dummy_projects() {
    let cfg: NoMistakesConfig = serde_yaml::from_str(
        r#"
testPlan:
  vitest:
    fullSuiteTriggers:
      - name: postgres-resources
        paths: [db/**/*.sql]
        targets: [backend]
        includeChangedTests: true
      - name: root-config
        paths: [package.json]
"#,
    )
    .unwrap();
    let triggers = &cfg.test_plan.vitest.full_suite_triggers.triggers;
    assert_eq!(triggers.len(), 2);
    assert_eq!(
        triggers[0],
        NamedFullSuiteTrigger {
            name: "postgres-resources".to_string(),
            paths: vec!["db/**/*.sql".to_string()],
            targets: vec!["backend".to_string()],
            include_changed_tests: Some(true),
        }
    );
    assert!(triggers[1].targets.is_empty());
    assert!(cfg.test_plan.vitest.full_suite_triggers.projects.is_empty());
}

#[test]
fn named_full_suite_triggers_object_form_accepts_triggers_key() {
    let cfg: NoMistakesConfig = serde_yaml::from_str(
        r#"
testPlan:
  vitest:
    fullSuiteTriggers:
      ignoreChangedTests: [playwright]
      triggers:
        - name: contracts
          paths: [api/openapi.json]
          targets: [backend]
"#,
    )
    .unwrap();
    assert_eq!(
        cfg.test_plan.vitest.full_suite_triggers.triggers[0].name,
        "contracts"
    );
}

#[test]
fn duplicate_named_trigger_names_are_rejected() {
    let result = serde_yaml::from_str::<NoMistakesConfig>(
        r#"
testPlan:
  vitest:
    fullSuiteTriggers:
      - name: dup
        paths: [a]
      - name: dup
        paths: [b]
"#,
    );
    assert!(result.unwrap_err().to_string().contains("duplicates"));
}

#[test]
fn named_trigger_names_and_paths_must_not_be_blank() {
    let blank_name = serde_yaml::from_str::<NoMistakesConfig>(
        r#"
testPlan:
  vitest:
    fullSuiteTriggers:
      - name: " "
        paths: [db/**]
"#,
    );
    assert!(blank_name
        .unwrap_err()
        .to_string()
        .contains("name must not be blank"));

    let empty_paths = serde_yaml::from_str::<NoMistakesConfig>(
        r#"
testPlan:
  vitest:
    fullSuiteTriggers:
      - name: resources
        paths: []
"#,
    );
    assert!(empty_paths
        .unwrap_err()
        .to_string()
        .contains("paths must not be empty"));
}

#[test]
fn changed_test_policy_requires_structured_named_targets() {
    let result = serde_yaml::from_str::<NoMistakesConfig>(
        r#"
testPlan:
  vitest:
    fullSuiteTriggers:
      - name: root-config
        paths: [package.json]
        includeChangedTests: false
"#,
    );
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("includeChangedTests requires non-empty targets"));
}

#[test]
fn playwright_coverage_gates_default_true_and_parse_false() {
    let default_cfg = NoMistakesConfig::default();
    assert!(default_cfg.tests.playwright.coverage.routes);
    assert!(default_cfg.tests.playwright.coverage.selectors);
    let cfg: NoMistakesConfig = serde_yaml::from_str(
        r#"
tests:
  playwright:
    coverage:
      routes: false
"#,
    )
    .unwrap();
    assert!(!cfg.tests.playwright.coverage.routes);
    assert!(cfg.tests.playwright.coverage.selectors);
}
