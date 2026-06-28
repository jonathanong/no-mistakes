use super::*;

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

    let report = analyze_contracts(&root, &route_report);

    assert_eq!(report.client_refs.len(), 2);
    assert!(report
        .client_refs
        .iter()
        .all(|client_ref| client_ref.query_params != vec!["debug"]));
    assert_eq!(report.mismatches.len(), 1);
    assert_eq!(report.mismatches[0].missing_params, vec!["sort"]);
}

#[test]
fn analyze_contracts_matches_same_path_routes_by_fetch_method() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/server-contracts/mismatch/fixture");
    let route_report = ProjectReport {
        summary: Default::default(),
        routes: vec![
            ServerRoute {
                file: "server.ts".to_string(),
                line: 1,
                method: "GET".to_string(),
                route: "/api/users".to_string(),
                raw_path: "/api/users".to_string(),
                query_params: vec!["sort".to_string()],
                framework: crate::server_routes::types::Framework::Express,
            },
            ServerRoute {
                file: "server.ts".to_string(),
                line: 2,
                method: "POST".to_string(),
                route: "/api/users".to_string(),
                raw_path: "/api/users".to_string(),
                query_params: vec!["include".to_string()],
                framework: crate::server_routes::types::Framework::Express,
            },
        ],
        edges: Vec::new(),
        diagnostics: Vec::new(),
    };

    let report = analyze_contracts(&root, &route_report);

    let mismatch = report
        .mismatches
        .iter()
        .find(|mismatch| mismatch.missing_params == vec!["sort"])
        .expect("POST fetch should be compared to POST route");
    assert_eq!(mismatch.matched_route, "/api/users");
}
