use super::test_support::*;
use super::*;
use std::path::Path;

#[test]
fn prepared_routes_and_contracts_share_one_union_fact_parse() {
    let source = crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/parser-count/server-contracts"),
    );
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    crate::ast::begin_parse_count(&root);

    let prepared = crate::server_routes::graph::prepare_analysis(&root, None).unwrap();
    let routes =
        crate::server_routes::graph::analyze_project_with_prepared(&prepared, &[]).unwrap();
    let contracts = analyze_contracts_with_prepared(&prepared, &routes, &[]).unwrap();
    let counts = crate::ast::finish_parse_count(&root);

    assert_eq!(routes.routes.len(), 1, "{routes:#?}");
    assert_eq!(contracts.client_refs.len(), 1, "{contracts:#?}");
    assert_eq!(counts.len(), prepared.source_files.len(), "{counts:#?}");
    assert!(
        prepared
            .source_files
            .iter()
            .all(|file| counts.get(file) == Some(&1)),
        "routes and contracts must share one parse per source: {counts:#?}"
    );

    let graph = include_str!("graph.rs");
    let graph_prepare = include_str!("graph_prepare.rs");
    let contracts_source = include_str!("contracts.rs");
    // Match the shared prefix so session-aware variants remain covered by the
    // aggregate-count guard instead of silently escaping it after a rename.
    assert_eq!(
        graph.matches("collect_ts_facts_with_context").count()
            + graph_prepare
                .matches("collect_ts_facts_with_context")
                .count(),
        2
    );
    assert!(graph.contains("route_refs: true") || graph_prepare.contains("route_refs: true"));
    assert!(graph.contains("server_routes: true") || graph_prepare.contains("server_routes: true"));
    assert!(!contracts_source.contains("collect_ts_facts_with_context"));
}

#[test]
fn source_file_filter_accepts_matching_root_relative_paths() {
    let root = Path::new("/repo");
    let filter = build_filter(&["src/routes/**/*.ts".to_string()])
        .expect("glob should compile")
        .expect("filter should be present");

    assert!(source_file_matches_filter(
        root,
        Path::new("/repo/src/routes/feed.ts"),
        Some(&filter),
    ));
    assert!(!source_file_matches_filter(
        root,
        Path::new("/repo/src/components/feed.ts"),
        Some(&filter),
    ));
}

#[test]
fn resolve_tsconfig_reports_explicit_relative_load_errors() {
    let error = resolve_tsconfig(Path::new("/repo"), Some(Path::new("missing-tsconfig.json")))
        .expect_err("explicit missing tsconfig should fail");

    assert!(error
        .to_string()
        .contains("loading tsconfig /repo/missing-tsconfig.json"));
}

#[test]
fn contracts_ignore_automatic_ignored_tsconfig_and_sources_but_honor_explicit_config() {
    let fixture = crate::test_support::materialize_gitignore_fixture("pass3-visibility");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    let visible = crate::codebase::ts_source::discover_visible_paths(fixture.path());

    let automatic = resolve_tsconfig_from_visible(fixture.path(), None, &visible).unwrap();
    assert!(automatic.paths.is_empty());
    let explicit =
        resolve_tsconfig_from_visible(fixture.path(), Some(Path::new("tsconfig.json")), &visible)
            .unwrap();
    assert!(!explicit.paths.is_empty());

    let route_report = ProjectReport {
        summary: Default::default(),
        routes: vec![ServerRoute {
            file: "server/router.ts".to_string(),
            line: 4,
            method: "GET".to_string(),
            route: "/visible".to_string(),
            raw_path: "/visible".to_string(),
            query_params: vec!["visible".to_string()],
            framework: crate::server_routes::types::Framework::Express,
        }],
        edges: Vec::new(),
        diagnostics: Vec::new(),
    };
    let report = analyze_contracts(fixture.path(), None, &route_report, &[]).unwrap();
    assert_eq!(report.client_refs.len(), 1);
    assert_eq!(report.client_refs[0].file, "client/visible-client.ts");
    assert_eq!(report.client_refs[0].query_params, vec!["visible"]);

    let prepared = crate::server_routes::graph::prepare_analysis(fixture.path(), None).unwrap();
    let reused = analyze_contracts_with_prepared(&prepared, &route_report, &[]).unwrap();
    assert_eq!(reused, report);
}

#[test]
fn contract_helpers_match_routes_and_extract_query_parts() {
    let report = ProjectReport {
        summary: Default::default(),
        routes: vec![ServerRoute {
            file: "backend/api/users.ts".to_string(),
            line: 12,
            method: "GET".to_string(),
            route: "/api/v1/users/:id".to_string(),
            raw_path: "/api/v1/users/:id".to_string(),
            query_params: vec!["include".to_string(), "page".to_string()],
            framework: crate::server_routes::types::Framework::Express,
        }],
        edges: Vec::new(),
        diagnostics: Vec::new(),
    };

    let matched = matching_route(&report, "/api/v1/users/123", None).unwrap();
    assert_eq!(matched.route, "/api/v1/users/:id");
    assert!(matching_route(&report, "/api/v1/projects/123", None).is_none());

    assert_eq!(
        query_params_from_pattern("/api/v1/users/123?page=2&include=posts&page=3#top").unwrap(),
        vec!["include", "page"]
    );
    assert!(query_params_from_pattern("/api/v1/users/123").is_none());
    assert_eq!(
        path_without_query("/api/v1/users/123?page=2#top"),
        "/api/v1/users/123"
    );
    assert_eq!(
        missing_query_params(
            &["include".to_string(), "sort".to_string()],
            &matched.query_params,
        ),
        vec!["sort"]
    );
}

#[test]
fn contract_helpers_ignore_dynamic_query_placeholders() {
    assert_eq!(
        query_params_from_pattern("/search?:param&term"),
        Some(vec!["term".to_string()])
    );
}

#[test]
fn analyze_contracts_reports_client_query_params_missing_from_server_route() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/server-contracts/mismatch/fixture");
    let route_report = ProjectReport {
        summary: Default::default(),
        routes: vec![ServerRoute {
            file: "server.ts".to_string(),
            line: 1,
            method: "GET".to_string(),
            route: "/api/users".to_string(),
            raw_path: "/api/users".to_string(),
            query_params: vec!["include".to_string()],
            framework: crate::server_routes::types::Framework::Express,
        }],
        edges: Vec::new(),
        diagnostics: Vec::new(),
    };

    let report = analyze_contracts(&root, None, &route_report, &[]).unwrap();

    assert_eq!(report.client_refs.len(), 3);
    assert!(report
        .client_refs
        .iter()
        .all(|client_ref| client_ref.query_params != vec!["debug"]));
    assert!(report
        .client_refs
        .iter()
        .any(|client_ref| client_ref.query_params == vec!["include"]));
    assert_eq!(report.mismatches.len(), 1);
    assert_eq!(report.mismatches[0].missing_params, vec!["sort"]);
}

include!("contracts_tests_methods.rs");
