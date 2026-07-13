use crate::fetch::types::{CacheKind, FetchOccurrence, FetchSide, SourceType};
use crate::playwright::analysis::fetch::{
    collect_fetches_for_routes_from_snapshot, expand_fetch_edges,
};
use crate::playwright::analysis::types::{Edge, FetchIndex};
use crate::playwright::test_support::fixture_path;
use crate::routes::Route;

fn server_fetch(path: &str) -> FetchOccurrence {
    FetchOccurrence {
        method: "GET".to_string(),
        path: path.to_string(),
        raw_path: path.to_string(),
        file: "web/app/page.tsx".to_string(),
        line: 1,
        side: FetchSide::Server,
        rsc: true,
        cached: false,
        cache_kind: CacheKind::None,
        cached_function: None,
        dynamic: false,
        unsupported: false,
        function_name: None,
        conditional: false,
        in_promise_all: false,
        error_handled: false,
        source_type: SourceType::Page,
    }
}

#[test]
fn collect_fetches_for_routes_surfaces_route_parse_errors() {
    let root = fixture_path(&["ast-snippets", "main", "invalid-route-fetch"]);
    let frontend_root = root.join("web/app");
    let routes = vec![Route {
        file: frontend_root.join("page.tsx"),
        pattern: "/".to_string(),
    }];

    let snapshot = crate::playwright::fsutil::VisiblePathSnapshot::new(&root);
    let err = collect_fetches_for_routes_from_snapshot(&routes, &frontend_root, &root, &snapshot)
        .unwrap_err();

    assert!(format!("{err:#}").contains("page.tsx"));
}

#[test]
fn expand_skips_non_route_edges() {
    let selector_edge = Edge::Selector {
        test_file: std::sync::Arc::new("tests/app.spec.ts".to_string()),
        test_name: None,
        describe_path: std::sync::Arc::new(vec![]),
        app_file: std::sync::Arc::new("web/app/page.tsx".to_string()),
        attribute: "data-testid".to_string(),
        value: "save".to_string(),
        selector: "getByTestId(save)".to_string(),
        line: 1,
    };
    let fetch_edge = Edge::Fetch {
        test_file: std::sync::Arc::new("tests/app.spec.ts".to_string()),
        test_name: None,
        describe_path: std::sync::Arc::new(vec![]),
        route_file: std::sync::Arc::new("web/app/page.tsx".to_string()),
        route: std::sync::Arc::new("/".to_string()),
        method: "GET".to_string(),
        path: "/api/health".to_string(),
        side: "server".to_string(),
        cached: false,
    };
    let mut index = FetchIndex::new();
    index.insert(
        "web/app/page.tsx".to_string(),
        vec![server_fetch("/api/health")],
    );
    let result = expand_fetch_edges(&[selector_edge, fetch_edge], &index);
    // Non-Route edges are skipped — no fetch expansion from them
    assert!(result.is_empty());
}

#[test]
fn expand_skips_routes_not_in_fetch_index() {
    let route_edge = Edge::Route {
        test_file: std::sync::Arc::new("tests/app.spec.ts".to_string()),
        test_name: None,
        describe_path: std::sync::Arc::new(vec![]),
        route_file: std::sync::Arc::new("web/app/missing/page.tsx".to_string()),
        route: std::sync::Arc::new("/missing".to_string()),
        url: std::sync::Arc::new("/missing".to_string()),
        hook: false,
        line: 1,
    };
    let index = FetchIndex::new();
    let result = expand_fetch_edges(&[route_edge], &index);
    assert!(result.is_empty());
}

#[test]
fn expand_skips_dynamic_and_unsupported_fetches() {
    let route_edge = Edge::Route {
        test_file: std::sync::Arc::new("tests/app.spec.ts".to_string()),
        test_name: None,
        describe_path: std::sync::Arc::new(vec![]),
        route_file: std::sync::Arc::new("web/app/page.tsx".to_string()),
        route: std::sync::Arc::new("/".to_string()),
        url: std::sync::Arc::new("/".to_string()),
        hook: false,
        line: 1,
    };
    let dynamic = FetchOccurrence {
        dynamic: true,
        unsupported: false,
        ..server_fetch("/api/dynamic")
    };
    let unsupported = FetchOccurrence {
        dynamic: false,
        unsupported: true,
        ..server_fetch("/api/unsupported")
    };
    let mut index = FetchIndex::new();
    index.insert("web/app/page.tsx".to_string(), vec![dynamic, unsupported]);
    let result = expand_fetch_edges(&[route_edge], &index);
    assert!(result.is_empty());
}

#[test]
fn expand_produces_client_side_fetch_edge() {
    let route_edge = Edge::Route {
        test_file: std::sync::Arc::new("tests/app.spec.ts".to_string()),
        test_name: Some(std::sync::Arc::new("clicks button".to_string())),
        describe_path: std::sync::Arc::new(vec!["Home".to_string()]),
        route_file: std::sync::Arc::new("web/app/page.tsx".to_string()),
        route: std::sync::Arc::new("/".to_string()),
        url: std::sync::Arc::new("/".to_string()),
        hook: false,
        line: 1,
    };
    let client_fetch = FetchOccurrence {
        method: "POST".to_string(),
        path: "/api/submit".to_string(),
        raw_path: "/api/submit".to_string(),
        file: "web/app/page.tsx".to_string(),
        line: 5,
        side: FetchSide::Client,
        rsc: false,
        cached: true,
        cache_kind: CacheKind::None,
        cached_function: None,
        dynamic: false,
        unsupported: false,
        function_name: None,
        conditional: false,
        in_promise_all: false,
        error_handled: false,
        source_type: SourceType::Page,
    };
    let mut index = FetchIndex::new();
    index.insert("web/app/page.tsx".to_string(), vec![client_fetch]);
    let result = expand_fetch_edges(&[route_edge], &index);
    assert_eq!(result.len(), 1);
    let Edge::Fetch {
        side,
        method,
        cached,
        test_name,
        describe_path,
        ..
    } = &result[0]
    else {
        panic!("expected Fetch edge");
    };
    assert_eq!(side, "client");
    assert_eq!(method, "POST");
    assert!(*cached);
    assert_eq!(
        test_name.as_deref().map(|s| s.as_str()),
        Some("clicks button")
    );
    assert_eq!(describe_path.as_slice(), &["Home".to_string()]);
}
