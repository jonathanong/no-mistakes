use crate::server_routes::{
    analyze_project, analyze_project_with_prepared, analyze_project_with_prepared_indexed,
    prepare_analysis_with_shared_facts, prepare_analysis_with_shared_facts_and_session,
    RelatedDirection,
};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/server-ast-routes")
        .join(name)
        .join("fixture")
}

#[test]
fn tsconfig_paths_resolve_mounted_routers() {
    let root = fixture("tsconfig-paths");
    let report = analyze_project(&root, Some(&root.join("tsconfig.json")), &[]).unwrap();
    let relative =
        analyze_project(&root, Some(std::path::Path::new("tsconfig.json")), &[]).unwrap();

    assert!(report
        .routes
        .iter()
        .any(|route| route.route == "/api/users/*" && route.file == "src/routes/users.ts"));
    assert!(report
        .routes
        .iter()
        .any(|route| route.route == "/cjs/ping" && route.file == "src/routes/common.cts"));
    assert_eq!(relative.routes, report.routes);
}

#[test]
fn implicit_invalid_tsconfig_falls_back_but_explicit_errors() {
    let root = fixture("invalid-tsconfig");

    let report = analyze_project(&root, None, &[]).unwrap();
    assert!(report.routes.iter().any(|route| route.route == "/health"));

    let err = analyze_project(&root, Some(&root.join("tsconfig.json")), &[]).unwrap_err();
    assert!(format!("{err:#}").contains("loading tsconfig"));
}

#[test]
fn malformed_v2_config_falls_back_to_unconfigured_scan() {
    let report = analyze_project(&fixture("malformed-config"), None, &[]).unwrap();

    assert!(report
        .routes
        .iter()
        .any(|route| route.route == "/api/users/*" && route.method == "get"));
}

#[test]
fn invalid_project_route_globs_fall_back_to_unconfigured_scan() {
    let report = analyze_project(&fixture("invalid-route-glob"), None, &[]).unwrap();

    assert!(report
        .routes
        .iter()
        .any(|route| route.route == "/api/users/*" && route.method == "get"));
}

#[test]
fn shared_fact_compatibility_preparation_matches_session_preparation() {
    let root = fixture("express").canonicalize().unwrap();
    let source_files = crate::codebase::ts_source::discover_files(&root, &[]);
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        paths: Vec::new(),
        base_url: None,
    };
    let config = crate::config::v2::NoMistakesConfig::default();
    let mut graph_context = crate::codebase::ts_source::facts::TsFactContext::new(&root);
    crate::server_routes::configure_fact_context(&mut graph_context, &root, &config);
    let shared = crate::codebase::check_facts::collect_check_facts(
        &root,
        source_files.clone(),
        crate::codebase::check_facts::CheckFactPlan {
            graph: crate::codebase::ts_source::facts::TsFactPlan {
                route_refs: true,
                server_routes: true,
                ..Default::default()
            },
            graph_context,
            ..Default::default()
        },
    );

    let compatibility =
        prepare_analysis_with_shared_facts(&root, &tsconfig, &config, &source_files, &shared);
    let session_aware = prepare_analysis_with_shared_facts_and_session(
        &root,
        &tsconfig,
        &config,
        &source_files,
        &shared,
        crate::codebase::analysis_session::AnalysisSession::disabled(),
    );
    assert!(compatibility.session.observer().is_none());

    let compatibility_report = analyze_project_with_prepared(&compatibility, &[]).unwrap();
    let session_report = analyze_project_with_prepared(&session_aware, &[]).unwrap();
    assert!(compatibility_report
        .routes
        .iter()
        .any(|route| route.route == "/api/v1/users/*" && route.method == "get"));
    assert_eq!(
        serde_json::to_value(&compatibility_report).unwrap(),
        serde_json::to_value(&session_report).unwrap()
    );

    let compatibility_indexed = analyze_project_with_prepared_indexed(&compatibility, &[]).unwrap();
    let session_indexed = analyze_project_with_prepared_indexed(&session_aware, &[]).unwrap();
    let roots = vec!["backend/api/users.ts".to_string()];
    assert_eq!(
        compatibility_indexed.edge_view(&roots, None),
        session_indexed.edge_view(&roots, None)
    );
    assert_eq!(
        compatibility_indexed.related(&roots, RelatedDirection::Both),
        session_indexed.related(&roots, RelatedDirection::Both)
    );
}
