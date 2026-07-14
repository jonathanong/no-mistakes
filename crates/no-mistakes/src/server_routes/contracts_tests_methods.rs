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

    let report = analyze_contracts(&root, None, &route_report, &[]).unwrap();

    let mismatch = report
        .mismatches
        .iter()
        .find(|mismatch| mismatch.missing_params == vec!["sort"])
        .expect("POST fetch should be compared to POST route");
    assert_eq!(mismatch.matched_route, "/api/users");
}

#[test]
fn analyze_contracts_applies_filters_to_client_scan() {
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
            query_params: vec!["include".to_string(), "sort".to_string()],
            framework: crate::server_routes::types::Framework::Express,
        }],
        edges: Vec::new(),
        diagnostics: Vec::new(),
    };

    let report = analyze_contracts(&root, None, &route_report, &["links.ts".to_string()]).unwrap();

    assert!(report.client_refs.is_empty());
    assert!(report.mismatches.is_empty());
}
