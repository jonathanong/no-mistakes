use super::*;

#[test]
fn shared_facts_path_matches_standalone_check() {
    let root = fixture("forbidden-dependencies-basic");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let graph_plan = graph_plan(&config).expect("fixture config enables forbidden dependencies");
    let (fact_plan, fact_context) =
        crate::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan(&root, graph_plan);
    let files =
        crate::codebase::ts_source::discover_files(&root, &config.filesystem.skip_directories);
    let shared = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            graph: fact_plan,
            graph_context: fact_context,
            ..Default::default()
        },
    );

    let standalone = check(&root, &config, None).unwrap();
    let with_facts = check_with_facts(&root, &config, None, None, &shared).unwrap();

    assert_eq!(with_facts, standalone);
}

#[test]
fn shared_facts_path_rejects_missing_graph_facts() {
    let root = fixture("forbidden-dependencies-basic");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let shared = crate::codebase::check_facts::CheckFactMap::default();

    let error = check_with_facts(&root, &config, None, None, &shared).unwrap_err();

    assert!(
        format!("{error:#}").contains("missing graph facts"),
        "expected missing graph facts error, got: {error:#}"
    );
}

#[test]
fn shared_facts_path_does_not_rediscover_when_graph_plan_needs_no_ts_facts() {
    let root = fixture("forbidden-dependencies-package-only");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let shared = crate::codebase::check_facts::CheckFactMap::default();

    assert!(
        check(&root, &config, None)
            .unwrap()
            .iter()
            .any(|finding| finding.rule == RULE_ID),
        "fixture must distinguish standalone discovery from the supplied empty universe"
    );
    let findings = check_with_facts(&root, &config, None, None, &shared).unwrap();

    assert!(
        findings.is_empty(),
        "shared analysis must not rediscover files outside its supplied universe: {findings:?}"
    );
}

#[test]
fn shared_facts_path_preserves_known_empty_graph_universe() {
    let root = fixture("forbidden-dependencies-package-only");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let shared = crate::codebase::check_facts::collect_check_facts_with_graph_files_and_playwright(
        &root,
        Vec::new(),
        Vec::new(),
        crate::codebase::check_facts::CheckFactPlan::default(),
        None,
    );

    assert!(shared.graph_file_universe_is_complete());
    assert!(shared.graph_file_universe().is_empty());
    assert!(
        check(&root, &config, None)
            .unwrap()
            .iter()
            .any(|finding| finding.rule == RULE_ID),
        "fixture must distinguish standalone discovery from a known-empty graph universe"
    );

    let findings = check_with_facts(&root, &config, None, None, &shared).unwrap();

    assert!(
        findings.is_empty(),
        "known-empty graph universe must not rediscover repository files: {findings:?}"
    );
}

#[test]
fn shared_facts_path_preserves_valid_findings_despite_parse_errors() {
    let root = fixture("forbidden-dependencies-parse-error");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let graph_plan = graph_plan(&config).expect("fixture config enables forbidden dependencies");
    let (fact_plan, fact_context) =
        crate::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan(&root, graph_plan);
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let shared = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            graph: fact_plan,
            graph_context: fact_context,
            ..Default::default()
        },
    );

    assert!(shared.stats.parse_errors > 0);
    let findings = check_with_facts(&root, &config, None, None, &shared).unwrap();

    assert!(
        findings.iter().any(|f| f.rule == RULE_ID),
        "valid cached facts must preserve the forbidden dependency finding: {findings:?}"
    );
}

#[test]
fn playwright_graph_consumers_do_not_reread_cached_parse_errors() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase/forbidden-playwright-cached-error/fixture"),
    );
    for (config_name, test_name, selector_consumer) in [
        ("route.no-mistakes.yml", "route.spec.ts", false),
        ("selector.no-mistakes.yml", "selector.spec.ts", true),
    ] {
        let config_path = root.join(config_name);
        let config = crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap();
        let graph_plan = graph_plan(&config).expect("forbidden graph plan");
        let (fact_plan, fact_context) =
            crate::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan_with_config(
                &root,
                graph_plan,
                Some(&config_path),
            );
        let graph_files = crate::codebase::dependencies::graph::GraphFiles::discover(&root)
            .all()
            .to_vec();
        let playwright = crate::playwright::rules::fact_plan_for_consumers(
            &root,
            Some(&config_path),
            &config,
            crate::playwright::rules::PlaywrightFactConsumers {
                graph_selectors: graph_plan.playwright_selectors,
                graph_routes: graph_plan.playwright_routes,
            },
        )
        .unwrap();
        let mut shared =
            crate::codebase::check_facts::collect_check_facts_with_graph_files_and_playwright(
                &root,
                graph_files.clone(),
                graph_files,
                crate::codebase::check_facts::CheckFactPlan {
                    graph: fact_plan,
                    graph_context: fact_context,
                    ..Default::default()
                },
                playwright,
            );
        let test_file = root.join("tests/e2e").join(test_name);
        let cached = shared.ts.get_mut(&test_file).expect("planned test facts");
        cached.playwright = None;
        // Disk deliberately disagrees with this cached error: a reread would create a finding.
        cached.parse_error = Some("cached Playwright parse error".to_string());
        shared.stats.parse_errors += 1;

        let standalone =
            super::super::check_with_config(&root, &config, Some(&config_path), None).unwrap();
        assert!(!standalone.is_empty(), "fixture must expose a reread");
        let findings = check_with_facts(&root, &config, Some(&config_path), None, &shared).unwrap();

        assert!(
            findings.is_empty(),
            "cached error must suppress disk reread"
        );
        if selector_consumer {
            assert_eq!(shared.app_selector_occurrences_cache.len(), 1);
        } else {
            assert_eq!(shared.playwright_routes_cache.len(), 1);
        }
    }
}

#[test]
fn shared_facts_path_propagates_graph_build_errors() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase/forbidden-playwright-cached-error/fixture"),
    );
    let config_path = root.join("selector.no-mistakes.yml");
    let config = crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap();
    let graph_plan = graph_plan(&config).expect("forbidden graph plan");
    let (fact_plan, fact_context) =
        crate::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan_with_config(
            &root,
            graph_plan,
            Some(&config_path),
        );
    let graph_files = crate::codebase::dependencies::graph::GraphFiles::discover(&root)
        .all()
        .to_vec();
    let shared = crate::codebase::check_facts::collect_check_facts_with_graph_files_and_playwright(
        &root,
        graph_files.clone(),
        graph_files,
        crate::codebase::check_facts::CheckFactPlan {
            graph: fact_plan,
            graph_context: fact_context,
            ..Default::default()
        },
        None,
    );
    let missing_config = root.join("missing.no-mistakes.yml");

    let error = check_with_facts(&root, &config, Some(&missing_config), None, &shared).unwrap_err();

    assert!(
        format!("{error:#}").contains("config file does not exist"),
        "expected graph build config error, got: {error:#}"
    );
}

#[test]
fn graph_plan_and_shared_facts_empty_when_rule_is_not_configured() {
    let root = fixture("forbidden-dependencies-basic");
    let config = NoMistakesConfig::default();
    let shared = crate::codebase::check_facts::CheckFactMap::default();

    assert!(graph_plan(&config).is_none());
    let findings = check_with_facts(&root, &config, None, None, &shared).unwrap();
    assert!(findings.is_empty());
}
