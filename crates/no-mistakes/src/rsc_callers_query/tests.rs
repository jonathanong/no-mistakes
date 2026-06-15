use super::*;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/rsc-callers/fixture")
}

fn run_button(depth: Option<usize>) -> RscCallersReport {
    run(
        &fixture(),
        None,
        None,
        Path::new("app/ui/Button.tsx"),
        depth,
    )
    .unwrap()
}

#[test]
fn reports_server_callers_and_prunes_client_boundary() {
    let report = run_button(None);
    assert_eq!(report.component, "app/ui/Button.tsx");

    let by_file: std::collections::HashMap<&str, &RscCaller> = report
        .callers
        .iter()
        .map(|caller| (caller.file.as_str(), caller))
        .collect();

    // Card (server component, depth 1) and ServerWidget (explicit server, depth 1).
    assert_eq!(by_file["app/ui/Card.tsx"].kind, CallerKind::Component);
    assert_eq!(by_file["app/ui/Card.tsx"].environment, Environment::Unknown);
    assert_eq!(by_file["app/ui/Card.tsx"].depth, 1);
    assert_eq!(
        by_file["app/ui/ServerWidget.tsx"].environment,
        Environment::Server
    );

    // Page reached transitively through Card at depth 2, classified as a page.
    assert_eq!(by_file["app/dashboard/page.tsx"].kind, CallerKind::Page);
    assert_eq!(by_file["app/dashboard/page.tsx"].depth, 2);

    // Client boundary excluded; its parent never reached; unrelated never imports.
    assert!(!by_file.contains_key("app/ui/ClientThing.tsx"));
    assert!(!by_file.contains_key("app/ui/ClientParent.tsx"));
    assert!(!by_file.contains_key("app/ui/unrelated.tsx"));
}

#[test]
fn depth_limit_excludes_transitive_page() {
    let report = run_button(Some(1));
    let files: Vec<&str> = report.callers.iter().map(|c| c.file.as_str()).collect();
    assert!(files.contains(&"app/ui/Card.tsx"));
    // page.tsx is at depth 2, beyond the depth-1 limit.
    assert!(!files.contains(&"app/dashboard/page.tsx"));
}

#[test]
fn paths_helper_returns_sorted_files() {
    let report = run_button(None);
    let paths = report.paths();
    assert!(paths.contains(&"app/ui/Card.tsx".to_string()));
    let mut sorted = paths.clone();
    sorted.sort();
    assert_eq!(paths, sorted);
}

#[test]
fn missing_component_has_no_callers() {
    let report = run(
        &fixture(),
        None,
        None,
        Path::new("app/ui/DoesNotExist.tsx"),
        None,
    )
    .unwrap();
    assert!(report.callers.is_empty());
}

#[test]
fn absolute_component_and_explicit_tsconfig() {
    let component = fixture().join("app/ui/Button.tsx");
    let tsconfig = fixture().join("tsconfig.json");
    let report = run(&fixture(), None, Some(&tsconfig), &component, None).unwrap();
    assert!(!report.callers.is_empty());
}

#[test]
fn detect_environment_variants() {
    assert_eq!(
        detect_environment(&fixture().join("app/ui/ServerWidget.tsx")),
        Environment::Server
    );
    assert_eq!(
        detect_environment(&fixture().join("app/ui/ClientThing.tsx")),
        Environment::Client
    );
    assert_eq!(
        detect_environment(&fixture().join("app/ui/Button.tsx")),
        Environment::Unknown
    );
    // Unreadable path falls back to Unknown.
    assert_eq!(
        detect_environment(Path::new("/no/such/component.tsx")),
        Environment::Unknown
    );
}

#[test]
fn resolve_tsconfig_falls_back_to_default() {
    let cfg = resolve_tsconfig(Path::new("/nonexistent-root"), None).unwrap();
    assert!(cfg.paths.is_empty());
}

#[test]
fn caller_kind_classifies_pages_and_components() {
    assert_eq!(
        caller_kind(Path::new("app/dashboard/page.tsx")),
        CallerKind::Page
    );
    assert_eq!(
        caller_kind(Path::new("app/ui/Card.tsx")),
        CallerKind::Component
    );
}
