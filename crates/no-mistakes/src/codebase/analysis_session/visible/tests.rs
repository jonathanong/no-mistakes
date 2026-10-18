use super::*;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/parser-count/signature-impact")
        .canonicalize()
        .unwrap()
}

#[test]
fn content_fingerprint_collisions_still_require_exact_membership() {
    let root = fixture_root();
    let visible = crate::fx::PathSet::from_iter([root.join("consumer.mts")]);
    let wrong = Arc::new(PreparedVisible::new(crate::fx::PathSet::from_iter([
        root.join("utils.mts")
    ])));
    let session = AnalysisSession::disabled();
    // Force a collision bucket, without depending on the hasher's current bits.
    session
        .visible_scopes()
        .scopes
        .insert(visible_fingerprint(&visible), vec![wrong.clone()]);
    let prepared = session.prepared_visible(&visible);
    assert!(!Arc::ptr_eq(&prepared, &wrong));
    assert!(prepared.contains_visible(&root.join("consumer.mts")));
    assert!(!prepared.contains_visible(&root.join("utils.mts")));
    assert!(Arc::ptr_eq(&prepared, &session.prepared_visible(&visible)));
}

#[test]
fn canonical_aliases_and_read_failures_are_shared_without_widening_raw_scopes() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/tsconfig/symlink-workspace");
    let lexical = crate::codebase::ts_resolver::normalize_path(&root.join("link/src/value.ts"));
    let real = lexical.canonicalize().unwrap();
    let missing = lexical.with_file_name("missing.ts");
    let visible = crate::fx::PathSet::from_iter([lexical.clone(), missing.clone()]);
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = AnalysisSession::new(Some(observer.clone()));
    let raw = session.prepared_visible(&visible);
    let normalized = session.normalized_visible(&visible);
    assert!(raw.contains_visible(&lexical));
    assert!(!raw.contains_visible(&real));
    assert!(normalized.contains_visible(&real));
    assert!(normalized.contains_visible(&missing));
    assert!(Arc::ptr_eq(
        &normalized,
        &session.normalized_visible(&visible)
    ));
    assert!(Arc::ptr_eq(
        &normalized,
        &session.normalized_visible(normalized.as_ref())
    ));
    let mut expected = vec![lexical, real, missing.clone()];
    expected.sort();
    assert_eq!(normalized.visible_cache_key(), expected);
    assert_eq!(
        observer.snapshot().work["resolver.visible_normalizations"],
        1
    );
    session.normalized_visible(&crate::fx::PathSet::from_iter([missing]));
    assert_eq!(
        observer.snapshot().work["resolver.visible_canonicalize_calls"],
        2
    );
}

#[test]
fn ordinary_paths_retain_no_duplicate_canonical_text_and_unused_requests_stay_lazy() {
    let root = fixture_root();
    let session = AnalysisSession::disabled();
    assert!(session.visible_scopes.get().is_none());
    session.read_source(&root.join("consumer.mts")).unwrap();
    assert!(session.visible_scopes.get().is_none());
    let visible =
        crate::fx::PathSet::from_iter([root.join("consumer.mts"), root.join("utils.mts")]);
    let prepared = session.normalized_visible(&visible);
    assert!(prepared.contains_visible(&root.join("utils.mts")));
    assert_eq!(session.visible_scopes().canonical.len(), 2);
    assert!(session
        .visible_scopes()
        .canonical
        .iter()
        .all(|entry| entry.value().is_none()));
}
