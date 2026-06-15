use super::*;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/effects/fixture")
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
fn missing_entry_is_empty() {
    let report = run(
        &fixture(),
        None,
        None,
        "valkey",
        Path::new("app/does-not-exist.ts"),
        &[],
        None,
    )
    .unwrap();
    assert!(report.call_sites.is_empty());
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
fn scan_file_ignores_unreadable_path() {
    let names = std::collections::HashMap::new();
    assert!(super::extract::scan_file(
        &fixture(),
        Path::new("/no/such/effects-file.ts"),
        0,
        &names
    )
    .is_empty());
}
