use crate::fetches::cli::Cli;
use crate::fetches::pipeline::run::{run_with_base_root, run_with_base_root_and_session};
use no_mistakes::cli::Format;
use std::path::PathBuf;

fn fixture(category: &str, name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases")
        .join(category)
        .join(name)
        .join("fixture")
}

#[test]
fn test_run_covers_paths_and_yml_formats() {
    let root = fixture("nextjs-fetches", "next-app");

    for format in [Format::Paths, Format::Yml] {
        let cli = Cli {
            root: root.clone(),
            config: None,
            format,
            json: false,
            targets: vec![],
        };

        crate::fetches::cli::run(cli).unwrap();
    }
}

#[cfg(feature = "test-instrumentation")]
#[test]
fn target_matching_and_fetch_analysis_parse_each_fixture_file_once() {
    let source = fixture("nextjs-fetches", "parse-sharing");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let cli = Cli {
        root: PathBuf::from("."),
        config: None,
        format: Format::Human,
        json: false,
        targets: vec!["app/users.ts".to_string()],
    };

    no_mistakes::ast::begin_parse_count(&root);
    let report = run_with_base_root(&root, &cli).unwrap();
    let counts = no_mistakes::ast::finish_parse_count(&root);

    assert_eq!(report.routes.len(), 1);
    assert_eq!(report.routes[0].route, "/");
    assert_eq!(report.routes[0].api_calls.len(), 1);
    assert_eq!(report.routes[0].api_calls[0].path, "/api/users");
    assert_eq!(counts.len(), 2);
    assert!(counts.values().all(|count| *count == 1), "{counts:?}");
}

#[test]
fn prepared_fetch_run_reuses_the_session_for_target_matching_and_traversal() {
    let root = fixture("nextjs-fetches", "parse-sharing");
    let cli = Cli {
        root: PathBuf::from("."),
        config: None,
        format: Format::Human,
        json: false,
        targets: vec!["app/users.ts".to_string()],
    };
    let observer = no_mistakes::diagnostics::InvocationObserver::new(true);
    let session = no_mistakes::codebase::analysis_session::AnalysisSession::new(Some(observer));

    let report = run_with_base_root_and_session(&root, &cli, &session).unwrap();

    assert_eq!(report.routes.len(), 1);
    assert_eq!(report.routes[0].api_calls.len(), 1);
    assert_eq!(report.routes[0].api_calls[0].path, "/api/users");
    let work = session.work_snapshot();
    let expected_paths = ["app/page.tsx", "app/users.ts"]
        .into_iter()
        .map(|path| no_mistakes::codebase::ts_resolver::normalize_path(&root.join(path)))
        .collect::<Vec<_>>();
    assert_eq!(
        work.source_reads.keys().cloned().collect::<Vec<_>>(),
        expected_paths,
        "target matching and traversal must read the expected prepared sources: {work:?}"
    );
    assert_eq!(
        work.parse_attempts.keys().cloned().collect::<Vec<_>>(),
        expected_paths,
        "target matching and traversal must parse the expected prepared sources: {work:?}"
    );
    assert!(
        work.source_reads.values().all(|count| *count == 1),
        "source reads must be memoized by the request session: {work:?}"
    );
    assert!(
        work.parse_attempts.values().all(|count| *count == 1),
        "target matching and traversal must share parsed facts: {work:?}"
    );
}

#[test]
fn test_run_with_base_root_errors_when_route_target_matcher_fails() {
    let root = fixture("nextjs-fetches", "route-target-matcher-parse-error");

    let cli = Cli {
        root: PathBuf::from("."),
        config: None,
        format: Format::Human,
        json: false,
        targets: vec!["app/target.ts".to_string()],
    };

    let err = run_with_base_root(&root, &cli).err().unwrap();
    assert!(err.to_string().contains("parse") || err.to_string().contains("expected"));
}

#[test]
fn test_run_with_base_root_errors_when_layout_target_matcher_fails() {
    let root = fixture("nextjs-fetches", "layout-target-matcher-parse-error");

    let cli = Cli {
        root: PathBuf::from("."),
        config: None,
        format: Format::Human,
        json: false,
        targets: vec!["app/target.ts".to_string()],
    };

    let err = run_with_base_root(&root, &cli).err().unwrap();
    assert!(err.to_string().contains("parse") || err.to_string().contains("expected"));
}

#[test]
fn test_run_with_base_root_errors_when_route_analysis_fails() {
    let root = fixture("nextjs-fetches", "route-analysis-parse-error");

    let cli = Cli {
        root: PathBuf::from("."),
        config: None,
        format: Format::Human,
        json: false,
        targets: vec![],
    };

    let err = run_with_base_root(&root, &cli).err().unwrap();
    assert!(err.to_string().contains("parse") || err.to_string().contains("expected"));
}

#[test]
fn test_run_with_base_root_errors_when_layout_analysis_fails() {
    let root = fixture("nextjs-fetches", "layout-analysis-parse-error");

    let cli = Cli {
        root: PathBuf::from("."),
        config: None,
        format: Format::Human,
        json: false,
        targets: vec![],
    };

    let err = run_with_base_root(&root, &cli).err().unwrap();
    assert!(err.to_string().contains("parse") || err.to_string().contains("expected"));
}

#[test]
fn test_run_with_base_root_errors_when_target_is_unmatched() {
    let root = fixture("nextjs-fetches", "app-page-fetch");

    let cli = Cli {
        root: PathBuf::from("."),
        config: None,
        format: Format::Human,
        json: false,
        targets: vec!["app/missing.ts".to_string()],
    };

    let err = run_with_base_root(&root, &cli).err().unwrap();
    let message = err.to_string();
    assert!(message.contains("Error: targets not found"));
    assert!(message.contains("app/missing.ts"));
}

#[test]
fn test_run_with_base_root_sorts_duplicates_and_unsupported_entries() {
    let root = fixture("nextjs-fetches", "sorts-duplicates-and-unsupported");

    let cli = Cli {
        root: PathBuf::from("."),
        config: None,
        format: Format::Human,
        json: false,
        targets: vec![],
    };
    let report = run_with_base_root(&root, &cli).unwrap();

    assert_eq!(report.unsupported.len(), 2);
    assert_eq!(report.unsupported[0].route, "/");
    assert!(report.unsupported[0].file.ends_with("app/page.tsx"));
    assert_eq!(report.unsupported[1].route, "/about");
    assert!(report.unsupported[1].file.ends_with("app/about/page.tsx"));

    assert_eq!(report.unsupported[0].reason, "dynamic-path");
    assert_eq!(report.unsupported[1].reason, "dynamic-path");

    assert_eq!(report.duplicates.len(), 2);
    assert_eq!(report.duplicates[0].key, "GET /api/duplicate server rsc");
    assert_eq!(report.duplicates[0].count, 2);
    assert_eq!(report.duplicates[1].key, "GET /api/other server rsc");
    assert_eq!(report.duplicates[1].count, 2);
}

#[test]
fn fetches_run_wrapper_publishes_the_next_app_fixture() {
    let cli = Cli {
        root: fixture("nextjs-fetches", "next-app"),
        config: None,
        format: Format::Json,
        json: true,
        targets: vec![],
    };
    let report = run_with_base_root(&std::env::current_dir().unwrap(), &cli).unwrap();
    let mut paths = report
        .routes
        .iter()
        .flat_map(|route| route.api_calls.iter().map(|call| call.path.clone()))
        .collect::<Vec<_>>();
    paths.sort();
    paths.dedup();
    assert!(
        paths.iter().any(|path| path == "/api/home"),
        "home page fetch missing: {paths:?}"
    );
    assert!(
        paths.iter().any(|path| path == "/api/users"),
        "imported users fetch missing: {paths:?}"
    );

    let mut published = Vec::new();
    crate::fetches::cli::publish_report_with_deadline_check(
        &report,
        Format::Json,
        &mut published,
        || Ok(()),
        || Ok(()),
    )
    .unwrap();
    let published = String::from_utf8(published).unwrap();
    assert!(published.contains("\"/api/home\""), "{published}");
    assert!(published.contains("\"/api/users\""), "{published}");
    assert!(published.contains("\"route\":\"/\""), "{published}");
    assert!(published.contains("\"route\":\"/users\""), "{published}");

    let code = crate::fetches::run(cli).unwrap();
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}

#[test]
fn target_file_match_uses_saved_layout_wrapper_chain() {
    // The layout file is the target. `reaches_target` returns true for that
    // same path before any import walk, so `wrapper_file == target_file` does
    // not change this report. The assertions pin the fetches that stay.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/nextjs-fetches/cli/fixture/target-file-match-uses-wrapper-chain");
    let report = run_with_base_root(
        &root,
        &Cli {
            root: PathBuf::from("."),
            config: None,
            format: Format::Human,
            json: false,
            targets: vec!["app/layout.tsx".into()],
        },
    )
    .unwrap();
    let routes = report
        .routes
        .iter()
        .map(|route| {
            let paths = route
                .api_calls
                .iter()
                .map(|call| call.path.as_str())
                .collect::<Vec<_>>()
                .join(",");
            format!("{} [{paths}]", route.route)
        })
        .collect::<Vec<_>>()
        .join("; ");
    let dashboard = report
        .routes
        .iter()
        .find(|route| route.route == "/dashboard")
        .unwrap_or_else(|| panic!("layout target dropped the dashboard page: {routes}"));
    assert!(
        dashboard
            .api_calls
            .iter()
            .any(|call| call.path == "/api/layout"),
        "layout file target must keep the layout fetch: {routes}"
    );
    assert!(
        dashboard
            .api_calls
            .iter()
            .any(|call| call.path == "/api/page"),
        "layout file target must keep the page fetch: {routes}"
    );
}
