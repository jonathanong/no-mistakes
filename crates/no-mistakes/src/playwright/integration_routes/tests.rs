use crate::playwright::{report_json, PlaywrightReportKind, PlaywrightReportOptions};
use std::path::PathBuf;

include!("coverage_tests.rs");

fn root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/playwright/integration-route-coverage"),
    )
}

fn options() -> PlaywrightReportOptions {
    PlaywrightReportOptions {
        root: root(),
        config: None,
        playwright_config: Vec::new(),
        project: None,
        app: None,
        files: Vec::new(),
        assert_conditional_tests: false,
        allow_skipped_tests: false,
        assert_unique_test_ids: false,
        assert_unique_html_ids: false,
    }
}

fn settings(
    snapshot: &crate::playwright::fsutil::VisiblePathSnapshot,
) -> crate::playwright::config::Settings {
    crate::playwright::config::load_settings_from_visible(&root(), None, &[], None, None, snapshot)
        .unwrap()
}

#[test]
fn omitted_sources_preserve_browser_edges_and_do_not_credit_integration_requests() {
    let root = root();
    let snapshot = crate::playwright::fsutil::VisiblePathSnapshot::new(&root);
    let enabled = settings(&snapshot);
    let mut omitted = enabled.clone();
    omitted.route_coverage_sources.clear();
    let analyze = |settings| {
        crate::playwright::analysis::pipeline_entrypoints::analyze_with_policy_from_snapshot(
            &root,
            settings,
            Default::default(),
            Default::default(),
            &snapshot,
        )
        .unwrap()
    };
    let enabled = analyze(&enabled);
    let omitted = analyze(&omitted);
    assert!(enabled
        .edges
        .edges
        .iter()
        .filter(|edge| !matches!(
            edge,
            crate::playwright::analysis::types::Edge::Route {
                attribution: Some(_),
                ..
            }
        ))
        .eq(omitted.edges.edges.iter()));
    let omitted = serde_json::to_value(omitted).unwrap();
    let covered: Vec<_> = omitted["coverage"]["routes"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|route| route["covered"] == true)
        .map(|route| route["route"].as_str().unwrap())
        .collect();
    assert_eq!(covered, ["/interactive"]);
}

#[test]
fn source_configuration_fails_closed_for_missing_owners_and_noncanonical_routes() {
    let root = root();
    let snapshot = crate::playwright::fsutil::VisiblePathSnapshot::new(&root);
    let settings = settings(&snapshot);
    for route in [
        "/health*",
        "/[id]",
        "https://example.com/healthz",
        "//healthz",
        "/healthz?q=1",
    ] {
        let mut sources = settings.route_coverage_sources.clone();
        sources[0].routes = vec![route.to_string()];
        assert!(super::validate(&sources).is_err(), "{route}");
    }
    for missing in ["project", "include", "routes", "helpers"] {
        let mut sources = settings.route_coverage_sources.clone();
        match missing {
            "project" => sources[0].project.clear(),
            "include" => sources[0].include.clear(),
            "routes" => sources[0].routes.clear(),
            _ => sources[0].helpers.clear(),
        }
        assert!(super::validate(&sources).is_err(), "{missing}");
    }
    for field in ["module", "export", "method"] {
        let mut sources = settings.route_coverage_sources.clone();
        match field {
            "module" => sources[0].helpers[0].module.clear(),
            "export" => sources[0].helpers[0].export.clear(),
            _ => sources[0].helpers[0].method = Some(String::new()),
        }
        assert!(super::validate(&sources).is_err(), "{field}");
    }
    let mut invalid_glob = settings.clone();
    invalid_glob.route_coverage_sources[0].include = vec!["[".to_string()];
    assert!(
        crate::playwright::analysis::pipeline_facts::standalone_fact_plan(
            &root,
            &invalid_glob,
            Default::default(),
            &snapshot,
        )
        .is_err()
    );
    for (project, route, expected) in [
        ("absent", "/healthz", "exactly one registered"),
        ("integration", "/absent", "not a canonical route"),
    ] {
        let mut invalid = settings.clone();
        invalid.route_coverage_sources[0].project = project.to_string();
        invalid.route_coverage_sources[0].routes = vec![route.to_string()];
        let error =
            crate::playwright::analysis::pipeline_entrypoints::analyze_with_policy_from_snapshot(
                &root,
                &invalid,
                Default::default(),
                Default::default(),
                &snapshot,
            )
            .err()
            .expect("invalid attribution must fail");
        assert!(error.to_string().contains(expected), "{error}");
    }
}

#[test]
fn graph_staging_preserves_precollected_route_occurrences_without_reparsing() {
    use crate::codebase::ts_source::facts::{TsFactMap, TsFactPlan};
    use std::sync::Arc;
    let root = root();
    let snapshot = crate::playwright::fsutil::VisiblePathSnapshot::new(&root);
    let settings = settings(&snapshot);
    let initial = crate::playwright::analysis::pipeline_facts::standalone_facts(
        &root,
        &settings,
        Default::default(),
        &snapshot,
    )
    .unwrap();
    let path = root.join("integration/cases.ts");
    let case = initial.ts.get(&path).unwrap();
    let graph_plan = TsFactPlan::imports();
    let precollected =
        TsFactMap::from_shared_iter_with_plan([(path.clone(), Arc::clone(&case.ts))], graph_plan);
    // A different retained value proves staging did not reread/reextract this
    // declaration from the fixture's actual `/healthz` request.
    let mut retained = case.integration_route_occurrences.clone();
    retained
        .iter_mut()
        .find(|fact| fact.occurrence.value == "/healthz")
        .unwrap()
        .occurrence
        .value = "/wrong-helper".to_string();
    let mut plan = crate::playwright::analysis::pipeline_facts::standalone_fact_plan(
        &root,
        &settings,
        Default::default(),
        &snapshot,
    )
    .unwrap();
    let sources = snapshot.source_store_for(&root);
    let tsconfig = crate::codebase::ts_resolver::resolve_tsconfig_from_visible_and_sources(
        None,
        &root,
        &snapshot.paths_for(&root),
        &sources,
    )
    .unwrap();
    plan.configure_module_resolution(
        Arc::new(tsconfig),
        Arc::new(Default::default()),
        &snapshot,
        &root,
    );
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(observer));
    let facts = crate::codebase::check_facts::collect_precollected_route_facts(
        &session,
        &root,
        (Vec::new(), snapshot.paths_for(&root).as_ref().clone(), true),
        crate::codebase::check_facts::CheckFactPlan {
            graph: graph_plan,
            ..Default::default()
        },
        plan,
        crate::codebase::check_facts::PrecollectedRouteFacts {
            ts: precollected,
            routes: [(path.clone(), retained)].into(),
        },
        sources,
    );
    assert!(!session.work_snapshot().parse_attempts.contains_key(&path));
    let links = facts
        .integration_route_links
        .values()
        .next()
        .unwrap()
        .as_ref()
        .unwrap();
    assert!(links
        .iter()
        .any(|link| link.occurrence.value == "/wrong-helper"));
    assert!(!links.iter().any(|link| link.occurrence.value == "/healthz"));
    assert!(links
        .iter()
        .all(|link| link.entry_file == root.join("integration/web.test.ts")));
}

#[test]
fn integration_routes_credit_only_registered_tests_with_bound_helpers_and_exact_routes() {
    let root = crate::codebase::ts_resolver::normalize_path(&root());
    let snapshot = crate::playwright::fsutil::VisiblePathSnapshot::new(&root);
    let settings = crate::playwright::config::load_settings_from_visible(
        &root,
        None,
        &[],
        None,
        None,
        &snapshot,
    )
    .unwrap();
    assert_eq!(settings.route_coverage_sources.len(), 1);
    let facts = crate::playwright::analysis::pipeline_facts::standalone_facts(
        &root,
        &settings,
        Default::default(),
        &snapshot,
    )
    .unwrap();
    let cases = facts.ts.get(&root.join("integration/cases.ts")).unwrap();
    assert_eq!(
        cases.integration_route_occurrences.len(),
        4,
        "imports: {:?}",
        cases.ts.imported_bindings
    );
    assert_eq!(
        facts
            .integration_route_links
            .values()
            .next()
            .unwrap()
            .as_ref()
            .unwrap()
            .len(),
        13
    );
    let report: serde_json::Value =
        serde_json::from_str(&report_json(PlaywrightReportKind::Check, options()).unwrap())
            .unwrap();
    let routes = report["routes"].as_array().unwrap();
    let covered: Vec<_> = routes
        .iter()
        .filter(|route| route["covered"] == true)
        .map(|route| route["route"].as_str().unwrap())
        .collect();
    assert_eq!(
        covered,
        ["/copyright", "/healthz", "/interactive", "/user/:id/posts"]
    );
    let health = routes
        .iter()
        .find(|route| route["route"] == "/healthz")
        .unwrap();
    assert_eq!(
        health["tests"],
        serde_json::json!(["integration/web.test.ts"])
    );
    assert_eq!(
        health["testsDetail"][0]["attribution"],
        serde_json::json!({
            "framework": "vitest", "project": "integration", "declarationFile": "integration/cases.ts",
        })
    );
    let interactive = routes
        .iter()
        .find(|route| route["route"] == "/interactive")
        .unwrap();
    assert!(interactive["testsDetail"][0].get("attribution").is_none());
}
