use super::*;

/// Regression test: `check_with_config` (used by both `run_check` and
/// `check_with_facts`'s missing-graph-universe fallback) must
/// resolve the `DepGraph`'s `GraphConfigOptions` from the given
/// `config_path`, not silently fall back to default discovery — `check`
/// itself delegates to `check_with_config(.., None, ..)`, so this also
/// covers that `check_with_config(.., Some(path), ..)` genuinely differs.
///
/// Reuses the `graph-default-route-config`/`graph-empty-route-config`
/// fixture pair from the graph module's own config_path tests: the former's
/// `.no-mistakes.yml` configures a real `backendPattern` (so `src/client.ts`
/// reaches `backend/api/users.mts` via a `RouteRef` edge), the latter's
/// configures an empty one (so it doesn't).
#[test]
fn check_with_config_uses_explicit_config_path() {
    let root = fixture("graph-default-route-config");
    let empty_config = fixture("graph-empty-route-config").join(".no-mistakes.yml");
    let raw = r#"
rules:
  - rule: forbidden-dependencies
    scope: repository
    options:
      roots:
        - src/client.ts
      forbiddenFiles:
        - backend/api/users.mts
      relationships:
        - route
"#;
    let config: NoMistakesConfig = serde_yaml::from_str(raw).unwrap();

    let default_findings = check_with_config(&root, &config, None, None).unwrap();
    assert!(
        !default_findings.is_empty(),
        "default-discovered config (this fixture's own .no-mistakes.yml) should register the backend route pattern and produce a finding via the RouteRef edge"
    );

    let explicit_findings = check_with_config(&root, &config, Some(&empty_config), None).unwrap();
    assert!(
        explicit_findings.is_empty(),
        "passing the explicit empty-pattern config must be honored, not silently ignored in favor of default discovery"
    );
}

#[test]
fn prepared_forbidden_dependencies_never_enters_legacy_discovery_fallback() {
    let source = include_str!("../shared.rs");
    let body = source
        .split("fn check_with_prepared_facts_and_graph(")
        .nth(1)
        .expect("prepared forbidden-dependencies body");

    assert!(!body.contains("return super::check_with_config"));
}

#[test]
fn legacy_graph_plan_keeps_default_fallback() {
    let config = NoMistakesConfig {
        rules: vec![crate::config::v2::schema::RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(crate::config::v2::schema::RuleScope::Repository),
            options: serde_yaml::from_str("relationships: false").unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    };

    assert!(graph_plan(&config).is_some());
    assert!(try_graph_plan(&config).is_err());
}
