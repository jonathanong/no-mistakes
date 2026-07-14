use super::*;

fn resolve_tsconfig(root: &Path, tsconfig: Option<&Path>) -> Result<TsConfig> {
    resolve_tsconfig_from_visible(
        root,
        tsconfig,
        &crate::codebase::ts_source::discover_visible_paths(root),
    )
}

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/effects/fixture")
}

fn parser_count_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/parser-count/effects")
}

fn gitignore_fixture() -> tempfile::TempDir {
    let fixture = crate::test_support::materialize_gitignore_fixture("prepared-tsconfig");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    fixture
}

fn run_kind(categories: &[String]) -> EffectsReport {
    run(
        &fixture(),
        None,
        None,
        "valkey",
        Path::new("app/server.ts"),
        categories,
        None,
    )
    .unwrap()
}

#[test]
fn collects_transitive_effect_call_sites() {
    let report = run_kind(&[]);
    assert_eq!(report.entry, "app/server.ts");

    let triples: Vec<(String, String, Option<String>, Option<String>)> = report
        .call_sites
        .iter()
        .map(|site| {
            (
                site.file.clone(),
                site.callee.clone(),
                site.category.clone(),
                site.caller.clone(),
            )
        })
        .collect();

    assert!(triples.contains(&(
        "lib/cache.ts".to_string(),
        "ValkeyCache".to_string(),
        Some("cache".to_string()),
        Some("makeCache".to_string())
    )));
    assert!(triples.contains(&(
        "lib/cache.ts".to_string(),
        "getEntityCache".to_string(),
        Some("cache".to_string()),
        Some("makeCache".to_string())
    )));
    assert!(triples.contains(&(
        "lib/pubsub.ts".to_string(),
        "createPublisher".to_string(),
        Some("pubsub".to_string()),
        Some("publish".to_string())
    )));
    assert!(triples.contains(&(
        "lib/b.ts".to_string(),
        "invalidate".to_string(),
        Some("invalidation".to_string()),
        Some("loop".to_string())
    )));

    // unused.ts is unreachable from the entry; its invalidate() is excluded.
    assert!(report.call_sites.iter().all(|s| s.file != "lib/unused.ts"));

    assert_eq!(report.by_category.get("cache"), Some(&2));
    assert_eq!(report.by_category.get("pubsub"), Some(&1));
    assert_eq!(report.by_category.get("invalidation"), Some(&1));
}

#[test]
fn category_filter_restricts_results() {
    let report = run_kind(&["pubsub".to_string()]);
    assert_eq!(report.call_sites.len(), 1);
    assert_eq!(report.call_sites[0].callee, "createPublisher");
    assert_eq!(report.paths(), vec!["lib/pubsub.ts"]);
}

#[test]
fn unknown_kind_errors_with_available_list() {
    let err = run(
        &fixture(),
        None,
        None,
        "bogus",
        Path::new("app/server.ts"),
        &[],
        None,
    )
    .unwrap_err();
    assert!(err.to_string().contains("unknown effects kind: bogus"));
    assert!(err.to_string().contains("valkey"));
}

#[test]
fn unknown_category_yields_no_functions() {
    let err = run(
        &fixture(),
        None,
        None,
        "valkey",
        Path::new("app/server.ts"),
        &["nope".to_string()],
        None,
    )
    .unwrap_err();
    assert!(err
        .to_string()
        .contains("no functions for the requested categories"));
}

#[test]
fn missing_entry_errors() {
    let err = run(
        &fixture(),
        None,
        None,
        "valkey",
        Path::new("app/does-not-exist.ts"),
        &[],
        None,
    )
    .unwrap_err();
    assert!(err.to_string().contains("entry file not found"));
}

#[test]
fn covers_arrow_caller_member_and_flat_functions() {
    // extra-entry reaches a file using an arrow-function caller, a member-call
    // effect, a parenthesized callee, and the flat `functions` list.
    let report = run(
        &fixture(),
        None,
        None,
        "valkey",
        Path::new("app/extra-entry.ts"),
        &[],
        None,
    )
    .unwrap();
    let triples: Vec<(String, Option<String>, Option<String>)> = report
        .call_sites
        .iter()
        .map(|s| (s.callee.clone(), s.category.clone(), s.caller.clone()))
        .collect();
    // Arrow caller `handler` attributed to the `new ValkeyCache()` site.
    assert!(triples.contains(&(
        "ValkeyCache".to_string(),
        Some("cache".to_string()),
        Some("handler".to_string())
    )));
    // Member call resolves by property name.
    assert!(triples.contains(&(
        "createSubscriber".to_string(),
        Some("pubsub".to_string()),
        Some("run".to_string())
    )));
    // Flat `functions` entry is uncategorized.
    assert!(triples.contains(&("standalone".to_string(), None, Some("run".to_string()))));
    assert_eq!(report.by_category.get("uncategorized"), Some(&1));
}

#[test]
fn absolute_entry_path_is_accepted() {
    let entry = fixture().join("app/server.ts");
    let report = run(&fixture(), None, None, "valkey", &entry, &[], None).unwrap();
    assert!(!report.call_sites.is_empty());
}

#[test]
fn ignored_automatic_tsconfig_does_not_resolve_effect_alias_but_explicit_does() {
    let fixture = gitignore_fixture();
    let automatic = run(
        fixture.path(),
        None,
        None,
        "regression",
        Path::new("effect-entry.ts"),
        &[],
        None,
    )
    .unwrap();
    assert!(automatic.call_sites.is_empty());

    let explicit = run(
        fixture.path(),
        None,
        Some(Path::new("tsconfig.json")),
        "regression",
        Path::new("effect-entry.ts"),
        &[],
        None,
    )
    .unwrap();
    assert_eq!(explicit.call_sites.len(), 1);
    assert_eq!(explicit.call_sites[0].file, "src/effect.ts");
}

#[test]
fn explicit_ignored_effect_entry_is_authoritative_but_ignored_transitive_is_not() {
    let fixture = gitignore_fixture();
    let report = run(
        fixture.path(),
        None,
        None,
        "regression",
        Path::new("ignored-explicit/effect-entry.ts"),
        &[],
        None,
    )
    .unwrap();

    assert!(
        report
            .call_sites
            .iter()
            .any(|site| site.file == "src/effect.ts"),
        "{report:#?}"
    );
    assert!(
        report
            .call_sites
            .iter()
            .all(|site| site.file != "ignored-transitive/effect.ts"),
        "{report:#?}"
    );
}

#[test]
fn explicit_tsconfig_is_honored() {
    let tsconfig = fixture().join("tsconfig.json");
    let report = run(
        &fixture(),
        None,
        Some(&tsconfig),
        "valkey",
        Path::new("app/server.ts"),
        &[],
        None,
    )
    .unwrap();
    assert!(!report.call_sites.is_empty());
}

#[test]
fn relative_tsconfig_resolved_against_root() {
    let report = run(
        &fixture(),
        None,
        Some(Path::new("tsconfig.json")),
        "valkey",
        Path::new("app/server.ts"),
        &[],
        None,
    )
    .unwrap();
    assert!(!report.call_sites.is_empty());
}

#[test]
fn unknown_kind_with_no_effects_config_lists_none() {
    // The data-pw fixture has no `effects` config at all.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/data-pw/fixture");
    let err = run(
        &root,
        None,
        None,
        "valkey",
        Path::new("app/search.tsx"),
        &[],
        None,
    )
    .unwrap_err();
    assert!(err.to_string().contains("<none>"));
}

#[test]
fn resolve_tsconfig_defaults_when_absent() {
    let cfg = resolve_tsconfig(Path::new("/nonexistent-effects-root"), None).unwrap();
    assert!(cfg.paths.is_empty());
}

#[test]
fn effects_reuses_one_parse_for_imports_and_effect_calls() {
    let source = crate::codebase::ts_resolver::normalize_path(&parser_count_fixture());
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    crate::ast::begin_parse_count(&root);

    let report = run(
        &root,
        None,
        None,
        "storage",
        Path::new("entry.ts"),
        &[],
        None,
    )
    .unwrap();
    let counts = crate::ast::finish_parse_count(&root);

    assert_eq!(report.call_sites.len(), 1, "{report:#?}");
    assert_eq!(report.call_sites[0].file, "effect.ts");
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

    let run_source = include_str!("../effects_query.rs");
    assert_eq!(
        run_source.matches("collect_ts_facts_with_context(").count(),
        1
    );
    assert!(!run_source.contains("scan_file("));
}
