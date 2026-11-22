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
fn canonical_graph_plans_include_forbidden_call_edges() {
    let mut config = crate::config::v2::NoMistakesConfig::default();
    config.rules.push(crate::config::v2::schema::RuleDef {
        rule: crate::codebase::rules::FORBIDDEN_CALLS.to_string(),
        scope: Some(crate::config::v2::schema::RuleScope::Repository),
        options: serde_yaml::from_str(
            "roots: [{ file: src/entry.mts }]\ntargets: [{ global: setTimeout }]",
        )
        .unwrap(),
        ..Default::default()
    });

    assert!(canonical_graph_plan(&config).is_some_and(|plan| plan.calls));
    assert!(try_canonical_graph_plan(&config)
        .unwrap()
        .is_some_and(|plan| plan.calls));
}

#[test]
fn canonical_graph_plans_include_calls_and_extends_for_unconstructed_error_class() {
    let mut config = crate::config::v2::NoMistakesConfig::default();
    config.rules.push(crate::config::v2::schema::RuleDef {
        rule: UNCONSTRUCTED_ERROR_CLASS.to_string(),
        scope: Some(crate::config::v2::schema::RuleScope::Repository),
        ..Default::default()
    });

    let calls_and_extends = |plan: GraphBuildPlan| plan.calls && plan.extends;
    assert!(canonical_graph_plan(&config).is_some_and(calls_and_extends));
    assert!(try_canonical_graph_plan(&config)
        .unwrap()
        .is_some_and(calls_and_extends));
}

#[test]
fn canonical_graph_plans_include_calls_and_imports_for_sql_cross_file_rules() {
    for rule in [
        crate::codebase::rules::POSTGRES_NO_MUTABLE_SQL_REGISTRY,
        crate::codebase::rules::POSTGRES_NO_UNBOUNDED_SQL_IDENTIFIER,
    ] {
        let mut config = crate::config::v2::NoMistakesConfig::default();
        config.rules.push(crate::config::v2::schema::RuleDef {
            rule: rule.to_string(),
            scope: Some(crate::config::v2::schema::RuleScope::Repository),
            options: serde_yaml::from_str("importSpecifier: '@example/db'").unwrap(),
            ..Default::default()
        });
        let plan = canonical_graph_plan(&config).expect(rule);
        assert!(plan.calls && plan.imports, "{rule}");
        assert!(super::canonical_graph_requires_full_file_universe(&config));
    }
}

#[test]
fn empty_fragment_tags_match_disabled_graph_demand() {
    use crate::config::v2::{load_v2_config, NoMistakesConfig};

    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-fragment-escape/configs");
    let load = |file: &str| load_v2_config(&directory, Some(&directory.join(file))).unwrap();
    let empty = load("empty-tags.yml");
    let active = load("append-chain.yml");
    let mut disabled = active.clone();
    disabled.rules[0].enabled = false;

    let baseline = NoMistakesConfig::default();
    assert!(canonical_graph_plan(&baseline).is_none());
    for config in [&disabled, &empty] {
        assert_eq!(
            canonical_graph_plan(config),
            canonical_graph_plan(&baseline)
        );
        assert_eq!(try_canonical_graph_plan(config).unwrap(), None);
        assert!(!canonical_graph_requires_full_file_universe(config));
    }

    let annotations = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/annotation-multi-root-shared-walks");
    for file in ["max.yml", "low.yml"] {
        let mut config = load_v2_config(&annotations, Some(&annotations.join(file))).unwrap();
        let plan = try_canonical_graph_plan(&config).unwrap();
        let full_universe = canonical_graph_requires_full_file_universe(&config);
        config.rules.extend(empty.rules.clone());
        assert_eq!(try_canonical_graph_plan(&config).unwrap(), plan, "{file}");
        assert_eq!(
            canonical_graph_requires_full_file_universe(&config),
            full_universe,
            "{file}"
        );
    }

    let required = try_canonical_graph_plan(&active).unwrap().unwrap();
    assert!(required.calls && required.imports);
    assert!(required.ts_fact_plan().needs_value_flow());
    assert!(canonical_graph_requires_full_file_universe(&active));
    let mut mixed = empty;
    mixed.rules.extend(active.rules);
    assert_eq!(try_canonical_graph_plan(&mixed).unwrap(), Some(required));
    assert!(canonical_graph_requires_full_file_universe(&mixed));
}

#[test]
fn malformed_fragment_options_keep_the_graph_plan_and_options_error() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-fragment-escape/configs");
    let mut config =
        crate::config::v2::load_v2_config(&directory, Some(&directory.join("empty-tags.yml")))
            .unwrap();
    config.rules[0].options["trustedSqlTags"] = serde_yaml::Value::String("invalid".to_string());
    config.rules[0].include.push("[".to_string());

    assert!(try_canonical_graph_plan(&config).unwrap().is_some());
    let error = crate::codebase::rules::postgres_sql_fragment_escape::check_with_graph(
        &directory,
        &config,
        None,
        &crate::codebase::check_facts::CheckFactMap::default(),
        None,
        None,
    )
    .err()
    .expect("malformed fragment options must fail");
    assert!(
        error.to_string().contains("options.trustedSqlTags"),
        "{error:#}"
    );
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
        prepared_vitest_projects: None,
        prepared_playwright_projects: None,
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
        prepared_vitest_projects: None,
        prepared_playwright_projects: None,
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

#[test]
fn production_export_rule_requests_shared_symbol_selectors_and_full_universe() {
    let mut config = crate::config::v2::NoMistakesConfig::default();
    config.rules.push(crate::config::v2::schema::RuleDef {
        rule: crate::codebase::rules::PRODUCTION_EXPORTS_ONLY_USED_BY_TESTS.to_string(),
        scope: Some(crate::config::v2::schema::RuleScope::Repository),
        ..Default::default()
    });
    let plan = try_canonical_graph_plan(&config).unwrap().unwrap();
    assert!(plan.symbol_import_selectors);
    assert!(plan.extends);
    assert!(plan.imports);
    assert!(plan.workspace);
    assert!(canonical_graph_requires_full_file_universe(&config));
    assert!(super::super::any_codebase_rule_enabled(&config));
}

#[test]
fn failed_graph_preparation_is_partial_or_strict_without_rebuilding_inputs() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check-consumer-isolation"),
    );
    let shared = crate::codebase::check_facts::CheckFactMap::default();
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let catalog =
        crate::codebase::ts_resolver::TsConfigCatalog::forced(&root, tsconfig.clone(), None);
    for malformed in [false, true] {
        let mut config = crate::config::v2::NoMistakesConfig::default();
        config.rules.push(crate::config::v2::schema::RuleDef {
            rule: if malformed {
                FORBIDDEN_DEPENDENCIES
            } else {
                UNCONSTRUCTED_ERROR_CLASS
            }
            .into(),
            scope: Some(crate::config::v2::schema::RuleScope::Repository),
            options: serde_yaml::from_str(if malformed {
                "relationships: invalid"
            } else {
                "{}"
            })
            .unwrap(),
            ..Default::default()
        });
        for partial in [false, true] {
            let session = crate::codebase::analysis_session::AnalysisSession::disabled();
            if partial {
                session.enable_partial_checks();
            }
            let output = run_check_with_config_and_facts_and_playwright(PreparedRulesCheck {
                session: session.clone(),
                root: &root,
                config_path: None,
                tsconfig_path: None,
                shared: &shared,
                prepared_playwright: None,
                config: &config,
                prepared_graph: None,
                prepared_tsconfig: &tsconfig,
                prepared_tsconfig_catalog: &catalog,
                prepared_vitest_projects: None,
                prepared_playwright_projects: None,
                inferred_roots: None,
                sources: None,
            });
            assert_eq!(output.is_ok(), partial, "{malformed}: {output:?}");
            if partial {
                assert!(output.unwrap().is_empty());
                let failures = session.check_failures();
                assert_eq!(failures.len(), 1);
                assert_eq!(failures[0].consumer, config.rules[0].rule);
                if !malformed {
                    assert!(failures[0].message.contains("missing graph facts"));
                }
            } else {
                assert!(session.check_failures().is_empty());
            }
        }
    }
}
