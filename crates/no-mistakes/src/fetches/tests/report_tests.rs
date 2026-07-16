use crate::fetches::report::print::{cache_kind_name, fetch_cache_label};
use crate::fetches::report::types::{
    ApiCallOccurrence, CacheKind, DuplicateApiCall, FetchOccurrence, FetchSide, FinalReport,
    RouteReport, SourceType, Summary, UnsupportedApiCall,
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
        function_name: None,
        conditional: false,
        in_promise_all: false,
        error_handled: false,
        source_type: SourceType::Page,
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
        function_name: None,
        conditional: false,
        in_promise_all: false,
        error_handled: false,
        source_type: SourceType::Page,
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
        function_name: None,
        conditional: false,
        in_promise_all: false,
        error_handled: false,
        source_type: SourceType::Page,
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
        function_name: None,
        conditional: false,
        in_promise_all: false,
        error_handled: false,
        source_type: SourceType::Page,
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
            function_name: None,
            conditional: false,
            in_promise_all: false,
            error_handled: false,
            source_type: SourceType::Page,
        }],
    };
    let serialized = serde_json::to_string(&report).unwrap();
    assert!(serialized.contains("\"apiCalls\""));
    assert!(!serialized.contains("\"api_calls\""));
}

#[test]
fn test_markdown_report_is_rendered() {
    use crate::fetches::report::print::write_markdown_report;

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
            conditional_api_calls: 0,
            parallel_api_calls: 0,
            error_handled_api_calls: 0,
        },
        routes: vec![
            RouteReport {
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
                    function_name: None,
                    conditional: false,
                    in_promise_all: false,
                    error_handled: false,
                    source_type: SourceType::Page,
                }],
            },
            RouteReport {
                route: "/empty".to_string(),
                file: "app/empty/page.tsx".to_string(),
                api_calls: Vec::new(),
            },
        ],
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

    let mut output = Vec::new();
    write_markdown_report(&report, &mut output).unwrap();
    let output = String::from_utf8(output).unwrap();
    assert!(output.starts_with("# Next.js Fetch API Analysis\n\n"));
    assert!(output.contains("## Duplicates"));
    assert!(output.contains("## Unsupported (Dynamic)"));

    struct FailAfter {
        remaining_writes: usize,
    }

    impl std::io::Write for FailAfter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.remaining_writes == 0 {
                return Err(std::io::Error::other("synthetic write failure"));
            }
            self.remaining_writes -= 1;
            Ok(bytes.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let mut completed = false;
    for remaining_writes in 0..512 {
        let mut writer = FailAfter { remaining_writes };
        if write_markdown_report(&report, &mut writer).is_ok() {
            assert!(remaining_writes > 0);
            completed = true;
            break;
        }
    }
    assert!(completed);
}

#[test]
fn fetch_report_timeout_discards_buffered_output_for_every_format() {
    use crate::fetches::cli::publish_report_with_deadline_check;
    use no_mistakes::cli::Format;

    for format in [
        Format::Json,
        Format::Yml,
        Format::Paths,
        Format::Md,
        Format::Human,
    ] {
        let mut output = Vec::new();
        let mut checks = 0;
        let error = publish_report_with_deadline_check(
            &FinalReport::default(),
            format,
            &mut output,
            || {
                checks += 1;
                if checks == 2 {
                    anyhow::bail!("synthetic timeout");
                }
                Ok(())
            },
        )
        .unwrap_err();

        assert!(error.to_string().contains("synthetic timeout"));
        assert!(output.is_empty());
        assert_eq!(checks, 2);
    }
}

#[test]
fn fetch_report_publication_errors_are_contextual() {
    struct FailingWriter;

    impl std::io::Write for FailingWriter {
        fn write(&mut self, _bytes: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("synthetic write failure"))
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let error = crate::fetches::cli::publish_report_with_deadline_check(
        &FinalReport::default(),
        no_mistakes::cli::Format::Json,
        &mut FailingWriter,
        || Ok(()),
    )
    .unwrap_err();

    assert!(error.to_string().contains("publishing fetch report"));
}

#[test]
fn test_source_type_label_covers_all_variants() {
    use crate::fetches::report::print::source_type_label;
    assert_eq!(source_type_label(&SourceType::Page), "page");
    assert_eq!(source_type_label(&SourceType::Layout), "layout");
    assert_eq!(source_type_label(&SourceType::Loading), "loading");
    assert_eq!(source_type_label(&SourceType::Error), "error");
    assert_eq!(source_type_label(&SourceType::Template), "template");
    assert_eq!(source_type_label(&SourceType::Route), "route");
    assert_eq!(source_type_label(&SourceType::Module), "module");
}
