use super::*;

#[test]
fn canonical_graph_plan_preserves_legacy_fallback_while_try_is_strict() {
    let mut config = crate::config::v2::NoMistakesConfig::default();
    config.rules.push(crate::config::v2::schema::RuleDef {
        rule: crate::codebase::rules::FORBIDDEN_DEPENDENCIES.to_string(),
        scope: Some(crate::config::v2::schema::RuleScope::Repository),
        options: serde_yaml::from_str("relationships: invalid").unwrap(),
        ..Default::default()
    });

    assert!(canonical_graph_plan(&config).is_some());
    assert!(try_canonical_graph_plan(&config).is_err());
}

#[test]
fn legacy_prepared_request_without_sources_uses_the_request_session() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check-runner/empty");
    let shared = crate::codebase::check_facts::CheckFactMap::default();
    let config = crate::config::v2::NoMistakesConfig::default();
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let catalog =
        crate::codebase::ts_resolver::TsConfigCatalog::forced(&root, tsconfig.clone(), None);

    let findings = run_check_with_config_and_facts_and_playwright(PreparedRulesCheck {
        session: crate::codebase::analysis_session::AnalysisSession::disabled(),
        root: &root,
        config_path: None,
        tsconfig_path: None,
        shared: &shared,
        prepared_playwright: None,
        config: &config,
        prepared_graph: None,
        prepared_tsconfig: &tsconfig,
        prepared_tsconfig_catalog: &catalog,
        inferred_roots: None,
        sources: None,
    })
    .unwrap();

    assert!(findings.is_empty());
}

#[test]
fn prepared_rules_collect_independent_rule_bodies_in_parallel() {
    let source = include_str!("execution.rs");
    let graph_start = source
        .find("let owned_graph;")
        .expect("canonical graph handling");
    let independent_start = source
        .find("independent::collect")
        .expect("independent rule collection");
    assert!(
        graph_start < independent_start,
        "canonical graph handling must stay before independent rules"
    );
    assert!(
        !source.contains("rayon::join"),
        "rule parallelism belongs in the independent collector"
    );
    let collector = include_str!("execution/independent.rs");
    assert!(
        collector.contains("rayon::join"),
        "independent rule bodies must collect in parallel"
    );
}

fn playwright_coverage_check(
    root: &std::path::Path,
    prepared_playwright: Option<&crate::playwright::rules::PreparedPlaywrightRules>,
) -> Vec<RuleFinding> {
    let config = crate::config::v2::load_v2_config(root, None).unwrap();
    assert!(
        crate::playwright::rules::configured(&config),
        "fixture must enable a Playwright rule"
    );
    let shared = crate::codebase::check_facts::CheckFactMap::default();
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.to_path_buf(),
        paths_dir: root.to_path_buf(),
        ..Default::default()
    };
    let catalog =
        crate::codebase::ts_resolver::TsConfigCatalog::forced(root, tsconfig.clone(), None);
    run_check_with_config_and_facts_and_playwright(PreparedRulesCheck {
        session: crate::codebase::analysis_session::AnalysisSession::disabled(),
        root,
        config_path: None,
        tsconfig_path: None,
        shared: &shared,
        prepared_playwright,
        config: &config,
        prepared_graph: None,
        prepared_tsconfig: &tsconfig,
        prepared_tsconfig_catalog: &catalog,
        inferred_roots: None,
        sources: None,
    })
    .unwrap()
}

#[test]
fn independent_playwright_runs_without_prepared_rules() {
    let root = crate::playwright::test_support::fixture_path(&["nextjs-coverage", "uncovered"]);
    let findings = playwright_coverage_check(&root, None);
    assert!(findings
        .iter()
        .any(|finding| finding.rule == crate::playwright::rules::PLAYWRIGHT_COVERAGE));
}

#[test]
fn independent_playwright_runs_with_prepared_rules() {
    let root = crate::playwright::test_support::fixture_path(&["nextjs-coverage", "uncovered"]);
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let prepared = crate::playwright::rules::prepare(&root, None, &config)
        .unwrap()
        .expect("uncovered fixture should prepare Playwright rules");
    let findings = playwright_coverage_check(&root, Some(&prepared));
    assert!(findings
        .iter()
        .any(|finding| finding.rule == crate::playwright::rules::PLAYWRIGHT_COVERAGE));
}
