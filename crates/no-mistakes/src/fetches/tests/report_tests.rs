use crate::fetches::report::print::{cache_kind_name, fetch_cache_label};
use crate::fetches::report::types::{
    ApiCallOccurrence, CacheKind, DuplicateApiCall, FetchOccurrence, FetchSide, FinalReport,
    RouteReport, Summary, UnsupportedApiCall,
};

#[test]
fn test_fetch_cache_kind_names() {
    assert_eq!(cache_kind_name(&CacheKind::None), "none");
    assert_eq!(cache_kind_name(&CacheKind::FetchCache), "fetch-cache");
    assert_eq!(
        cache_kind_name(&CacheKind::FetchNextRevalidate),
        "fetch-next-revalidate"
    );
    assert_eq!(
        cache_kind_name(&CacheKind::FetchNextTags),
        "fetch-next-tags"
    );
    assert_eq!(cache_kind_name(&CacheKind::ReactCache), "react-cache");
    assert_eq!(cache_kind_name(&CacheKind::Cache), "cache");
    assert_eq!(cache_kind_name(&CacheKind::UnstableCache), "unstable-cache");
}

#[test]
fn test_fetch_cache_label_includes_cached_function() {
    let fetch = FetchOccurrence {
        path: "/api/example".to_string(),
        raw_path: "/api/example".to_string(),
        method: "GET".to_string(),
        file: "app/page.tsx".to_string(),
        line: 1,
        side: FetchSide::Server,
        rsc: true,
        cached: true,
        cache_kind: CacheKind::ReactCache,
        cached_function: Some("cache".to_string()),
        dynamic: false,
        unsupported: false,
    };
    assert_eq!(fetch_cache_label(&fetch), "react-cache (cache)");
}

#[test]
fn test_fetch_cache_label_without_cached_function() {
    let fetch = FetchOccurrence {
        path: "/api/example".to_string(),
        raw_path: "/api/example".to_string(),
        method: "GET".to_string(),
        file: "app/page.tsx".to_string(),
        line: 1,
        side: FetchSide::Server,
        rsc: true,
        cached: true,
        cache_kind: CacheKind::FetchCache,
        cached_function: None,
        dynamic: false,
        unsupported: false,
    };
    assert_eq!(fetch_cache_label(&fetch), "fetch-cache");
}

#[test]
fn test_fetch_cache_label_with_next_revalidate_kind() {
    let fetch = FetchOccurrence {
        path: "/api/example".to_string(),
        raw_path: "/api/example".to_string(),
        method: "GET".to_string(),
        file: "app/page.tsx".to_string(),
        line: 1,
        side: FetchSide::Server,
        rsc: true,
        cached: true,
        cache_kind: CacheKind::FetchNextRevalidate,
        cached_function: None,
        dynamic: false,
        unsupported: false,
    };
    assert_eq!(fetch_cache_label(&fetch), "fetch-next-revalidate");
}

#[test]
fn test_fetch_cache_label_with_next_tags_kind() {
    let fetch = FetchOccurrence {
        path: "/api/example".to_string(),
        raw_path: "/api/example".to_string(),
        method: "GET".to_string(),
        file: "app/page.tsx".to_string(),
        line: 1,
        side: FetchSide::Server,
        rsc: true,
        cached: true,
        cache_kind: CacheKind::FetchNextTags,
        cached_function: None,
        dynamic: false,
        unsupported: false,
    };
    assert_eq!(fetch_cache_label(&fetch), "fetch-next-tags");
}

#[test]
fn test_route_report_api_calls_uses_camel_case() {
    let report = RouteReport {
        route: "/".to_string(),
        file: "app/page.tsx".to_string(),
        api_calls: vec![FetchOccurrence {
            path: "/api/example".to_string(),
            raw_path: "/api/example".to_string(),
            method: "GET".to_string(),
            file: "app/page.tsx".to_string(),
            line: 3,
            side: FetchSide::Server,
            rsc: true,
            cached: false,
            cache_kind: CacheKind::None,
            cached_function: None,
            dynamic: false,
            unsupported: false,
        }],
    };
    let serialized = serde_json::to_string(&report).unwrap();
    assert!(serialized.contains("\"apiCalls\""));
    assert!(!serialized.contains("\"api_calls\""));
}

#[test]
fn test_print_markdown_report_is_rendered() {
    use crate::fetches::report::print::print_markdown_report;

    let report = FinalReport {
        summary: Summary {
            total_routes: 1,
            routes_with_api_calls: 1,
            total_api_calls: 1,
            unique_api_calls: 1,
            duplicate_api_calls: 0,
            dynamic_api_calls: 0,
            cached_api_calls: 0,
            client_api_calls: 0,
            server_api_calls: 1,
            rsc_api_calls: 1,
        },
        routes: vec![RouteReport {
            route: "/".to_string(),
            file: "app/page.tsx".to_string(),
            api_calls: vec![FetchOccurrence {
                path: "/api/page".to_string(),
                raw_path: "/api/page".to_string(),
                method: "GET".to_string(),
                file: "app/page.tsx".to_string(),
                line: 1,
                side: FetchSide::Server,
                rsc: true,
                cached: false,
                cache_kind: CacheKind::None,
                cached_function: None,
                dynamic: false,
                unsupported: false,
            }],
        }],
        duplicates: vec![DuplicateApiCall {
            key: "GET /api/page server rsc".to_string(),
            count: 2,
            occurrences: vec![
                ApiCallOccurrence {
                    route: "/".to_string(),
                    file: "app/page.tsx".to_string(),
                    line: 1,
                },
                ApiCallOccurrence {
                    route: "/about".to_string(),
                    file: "app/about/page.tsx".to_string(),
                    line: 2,
                },
            ],
        }],
        unsupported: vec![UnsupportedApiCall {
            route: "/".to_string(),
            file: "app/page.tsx".to_string(),
            line: 3,
            reason: "dynamic-path".to_string(),
            raw_path: "fetch(url)".to_string(),
        }],
    };

    print_markdown_report(&report);
}
