use crate::fetches::pipeline::run::run_with_base_root;
use no_mistakes::cli::Format;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/nextjs-fetches")
        .join(name)
        .join("fixture")
}

fn cli(root: PathBuf) -> crate::fetches::cli::Cli {
    crate::fetches::cli::Cli {
        root,
        config: None,
        format: Format::Json,
        json: true,
        targets: vec![],
    }
}

fn find_fetch<'a>(
    report: &'a crate::fetches::report::types::FinalReport,
    route: &str,
    path: &str,
) -> &'a no_mistakes::fetch::types::FetchOccurrence {
    report
        .routes
        .iter()
        .find(|r| r.route == route)
        .unwrap_or_else(|| panic!("route {route} not found"))
        .api_calls
        .iter()
        .find(|f| f.path == path)
        .unwrap_or_else(|| panic!("fetch {path} not found in route {route}"))
}

#[test]
fn metadata_context_function_names() {
    let root = fixture("metadata-context");
    let report = run_with_base_root(&root, &cli(root.clone())).unwrap();
    assert_eq!(
        find_fetch(&report, "/", "/api/unconditional").function_name,
        Some("HomePage".to_string())
    );
    assert_eq!(
        find_fetch(&report, "/", "/api/layout-data").function_name,
        Some("RootLayout".to_string())
    );
    assert_eq!(
        find_fetch(&report, "/api/data", "/api/external").function_name,
        Some("GET".to_string())
    );
    assert_eq!(
        find_fetch(&report, "/dashboard", "/api/users").function_name,
        Some("getUsers".to_string())
    );
    assert_eq!(
        find_fetch(&report, "/dashboard", "/api/posts").function_name,
        Some("getPosts".to_string())
    );
}

#[test]
fn metadata_context_conditional() {
    let root = fixture("metadata-context");
    let report = run_with_base_root(&root, &cli(root.clone())).unwrap();
    assert!(!find_fetch(&report, "/", "/api/unconditional").conditional);
    assert!(find_fetch(&report, "/", "/api/conditional").conditional);
    assert!(find_fetch(&report, "/", "/api/ternary-a").conditional);
    assert!(find_fetch(&report, "/", "/api/ternary-b").conditional);
}

#[test]
fn metadata_context_promise_all() {
    let root = fixture("metadata-context");
    let report = run_with_base_root(&root, &cli(root.clone())).unwrap();
    assert!(find_fetch(&report, "/", "/api/parallel-1").in_promise_all);
    assert!(find_fetch(&report, "/", "/api/parallel-2").in_promise_all);
    assert!(!find_fetch(&report, "/", "/api/unconditional").in_promise_all);
}

#[test]
fn metadata_context_error_handled() {
    let root = fixture("metadata-context");
    let report = run_with_base_root(&root, &cli(root.clone())).unwrap();
    assert!(find_fetch(&report, "/", "/api/parallel-1").error_handled);
    assert!(!find_fetch(&report, "/", "/api/unconditional").error_handled);
}

#[test]
fn metadata_context_source_types() {
    use no_mistakes::fetch::types::SourceType;
    let root = fixture("metadata-context");
    let report = run_with_base_root(&root, &cli(root.clone())).unwrap();
    assert_eq!(
        find_fetch(&report, "/", "/api/unconditional").source_type,
        SourceType::Page
    );
    assert_eq!(
        find_fetch(&report, "/", "/api/layout-data").source_type,
        SourceType::Layout
    );
    assert_eq!(
        find_fetch(&report, "/", "/api/loading-data").source_type,
        SourceType::Loading
    );
    assert_eq!(
        find_fetch(&report, "/", "/api/error-data").source_type,
        SourceType::Error
    );
    assert_eq!(
        find_fetch(&report, "/", "/api/template-data").source_type,
        SourceType::Template
    );
    assert_eq!(
        find_fetch(&report, "/api/data", "/api/external").source_type,
        SourceType::Route
    );
    assert_eq!(
        find_fetch(&report, "/dashboard", "/api/users").source_type,
        SourceType::Module
    );
}

#[test]
fn error_handling_patterns_try_catch_vs_finally() {
    let root = fixture("error-handling-patterns");
    let report = run_with_base_root(&root, &cli(root.clone())).unwrap();
    assert!(find_fetch(&report, "/", "/api/handled").error_handled);
    assert!(!find_fetch(&report, "/", "/api/finally-only").error_handled);
    assert!(!find_fetch(&report, "/", "/api/unhandled").error_handled);
}

#[test]
fn logical_conditional_operators() {
    let root = fixture("logical-conditional");
    let report = run_with_base_root(&root, &cli(root.clone())).unwrap();
    assert!(find_fetch(&report, "/", "/api/and-right").conditional);
    assert!(find_fetch(&report, "/", "/api/or-right").conditional);
    assert!(find_fetch(&report, "/", "/api/nullish-right").conditional);
    assert!(!find_fetch(&report, "/", "/api/unconditional").conditional);
}

#[test]
fn promise_all_settled_detected() {
    let root = fixture("promise-all-patterns");
    let report = run_with_base_root(&root, &cli(root.clone())).unwrap();
    assert!(find_fetch(&report, "/", "/api/settled-1").in_promise_all);
    assert!(find_fetch(&report, "/", "/api/settled-2").in_promise_all);
    assert!(!find_fetch(&report, "/", "/api/sequential").in_promise_all);
}

#[test]
fn function_name_patterns() {
    let root = fixture("function-name-patterns");
    let report = run_with_base_root(&root, &cli(root.clone())).unwrap();
    assert_eq!(
        find_fetch(&report, "/", "/api/arrow-fn").function_name,
        Some("getData".to_string())
    );
    assert_eq!(
        find_fetch(&report, "/", "/api/named-fn").function_name,
        Some("loadItems".to_string())
    );
    assert_eq!(
        find_fetch(&report, "/", "/api/component").function_name,
        Some("Page".to_string())
    );
}

#[test]
fn summary_counts_new_dimensions() {
    let root = fixture("metadata-context");
    let report = run_with_base_root(&root, &cli(root.clone())).unwrap();
    assert!(report.summary.conditional_api_calls > 0);
    assert!(report.summary.parallel_api_calls > 0);
    assert!(report.summary.error_handled_api_calls > 0);
}
