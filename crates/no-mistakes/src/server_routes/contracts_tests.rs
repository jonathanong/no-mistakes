use super::*;

#[test]
fn contract_helpers_match_routes_and_extract_query_parts() {
    let report = ProjectReport {
        summary: Default::default(),
        routes: vec![
            ServerRoute {
                file: "backend/api/users.ts".to_string(),
                line: 12,
                method: "GET".to_string(),
                route: "/api/v1/users/:id".to_string(),
                raw_path: "/api/v1/users/:id".to_string(),
                query_params: vec!["include".to_string(), "page".to_string()],
                framework: crate::server_routes::types::Framework::Express,
            },
            ServerRoute {
                file: "backend/api/search.ts".to_string(),
                line: 20,
                method: "GET".to_string(),
                route: "/api/v1/users/search".to_string(),
                raw_path: "/api/v1/users/search".to_string(),
                query_params: vec!["term".to_string()],
                framework: crate::server_routes::types::Framework::Express,
            },
        ],
        edges: Vec::new(),
        diagnostics: Vec::new(),
    };

    let matched = matching_route(&report, "/api/v1/users/123").unwrap();
    assert_eq!(matched.route, "/api/v1/users/:id");
    let static_matched = matching_route(&report, "/api/v1/users/search").unwrap();
    assert_eq!(static_matched.route, "/api/v1/users/search");
    assert!(matching_route(&report, "/api/v1/projects/123").is_none());

    assert_eq!(
        query_params_from_pattern("/api/v1/users/123?page=2&include=posts&page=3#top").unwrap(),
        vec!["include", "page"]
    );
    assert!(query_params_from_pattern("/api/v1/users/123").is_none());
    assert!(query_params_from_pattern("/api/v1/users/123?:param").is_none());
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
