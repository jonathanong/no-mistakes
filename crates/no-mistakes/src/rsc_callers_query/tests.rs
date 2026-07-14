use super::test_support::*;
use super::*;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/rsc-callers/fixture")
}

fn parser_count_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/parser-count/rsc")
}

#[test]
fn rsc_callers_reuses_one_parse_for_imports_and_directives() {
    let source = crate::codebase::ts_resolver::normalize_path(&parser_count_fixture());
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    crate::ast::begin_parse_count(&root);

    let report = run(&root, None, None, Path::new("app/Target.tsx"), None).unwrap();
    let counts = crate::ast::finish_parse_count(&root);

    assert_eq!(report.callers.len(), 2, "{report:#?}");
    assert_eq!(report.callers[0].file, "app/ServerWidget.tsx");
    assert_eq!(report.callers[1].file, "app/page.tsx");
    let files = crate::codebase::dependencies::graph::GraphFiles::from_files(
        crate::codebase::ts_source::discover_files(&root, &[]),
    )
    .indexable()
    .to_vec();
    assert_eq!(counts.len(), files.len(), "{counts:#?}");
    assert!(
        files.iter().all(|file| counts.get(file) == Some(&1)),
        "each source file must be parsed once: {counts:#?}"
    );

    let run_source = include_str!("prepare.rs");
    assert_eq!(
        run_source.matches("collect_ts_facts_with_context(").count(),
        1
    );
    let run_body = run_source
        .split("pub fn run(")
        .nth(1)
        .and_then(|body| body.split("pub(crate) fn run_with_prepared").next())
        .expect("run body");
    assert!(!run_body.contains("detect_environment("));
}

fn gitignore_fixture() -> tempfile::TempDir {
    let fixture = crate::test_support::materialize_gitignore_fixture("prepared-tsconfig");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    fixture
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
fn depth_zero_reports_no_callers() {
    // `--depth 0` is a hard limit: not even direct importers are reported.
    let report = run_button(Some(0));
    assert!(report.callers.is_empty());
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
fn missing_component_errors() {
    let err = run(
        &fixture(),
        None,
        None,
        Path::new("app/ui/DoesNotExist.tsx"),
        None,
    )
    .unwrap_err();
    assert!(err.to_string().contains("component file not found"));
}

#[test]
fn existing_unimported_component_has_no_callers() {
    let report = run(&fixture(), None, None, Path::new("app/ui/Orphan.tsx"), None).unwrap();
    assert!(report.callers.is_empty());
}

#[test]
fn rsc_callers_ignore_automatic_ignored_tsconfig_but_honor_explicit_path() {
    let fixture = gitignore_fixture();
    let automatic = run(
        fixture.path(),
        None,
        None,
        Path::new("src/Button.tsx"),
        None,
    )
    .unwrap();
    assert!(automatic.callers.is_empty());

    let explicit = run(
        fixture.path(),
        None,
        Some(Path::new("tsconfig.json")),
        Path::new("src/Button.tsx"),
        None,
    )
    .unwrap();
    assert!(explicit
        .callers
        .iter()
        .any(|caller| caller.file == "stories/Button.stories.tsx"));
}

#[test]
fn explicit_ignored_component_is_authoritative_for_visible_importers() {
    let fixture = gitignore_fixture();
    let report = run(
        fixture.path(),
        None,
        None,
        Path::new("ignored-explicit/Button.tsx"),
        None,
    )
    .unwrap();

    assert!(
        report
            .callers
            .iter()
            .any(|caller| caller.file == "src/IgnoredButtonUser.tsx"),
        "{report:#?}"
    );
}

#[test]
fn absolute_component_and_explicit_tsconfig() {
    let component = fixture().join("app/ui/Button.tsx");
    let tsconfig = fixture().join("tsconfig.json");
    let report = run(&fixture(), None, Some(&tsconfig), &component, None).unwrap();
    assert!(!report.callers.is_empty());
}

#[test]
fn relative_tsconfig_resolved_against_root() {
    let report = run(
        &fixture(),
        None,
        Some(Path::new("tsconfig.json")),
        Path::new("app/ui/Button.tsx"),
        None,
    )
    .unwrap();
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
