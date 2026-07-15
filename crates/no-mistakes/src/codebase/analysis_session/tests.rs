use super::*;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/parser-count/signature-impact")
}

#[test]
fn source_success_and_failure_are_memoized() {
    let observer = InvocationObserver::new(true);
    let session = AnalysisSession::new(Some(Arc::clone(&observer)));
    let source = fixture_root().join("consumer.mts");
    let missing = fixture_root().join("missing.mts");

    assert_eq!(
        session.read_source(&source).unwrap(),
        session.read_source(&source).unwrap()
    );
    assert_eq!(
        session.read_source(&missing).unwrap_err(),
        session.read_source(&missing).unwrap_err()
    );

    let work = session.work_snapshot();
    assert_eq!(work.source_reads[&normalize_path(&source)], 1);
    assert_eq!(work.source_reads[&normalize_path(&missing)], 1);
    let diagnostics = observer.snapshot();
    assert_eq!(diagnostics.work["source.requests"], 4);
    assert_eq!(diagnostics.work["source.reads"], 2);
    assert_eq!(diagnostics.work["source.cache_hits"], 2);
    assert_eq!(diagnostics.work["source.read_errors"], 1);
}

#[test]
fn discovery_is_memoized_by_normalized_root() {
    let observer = InvocationObserver::new(true);
    let session = AnalysisSession::new(Some(Arc::clone(&observer)));
    let root = fixture_root();

    let first = session.visible_paths(&root);
    let second = session.visible_paths(&root.join("."));

    assert!(Arc::ptr_eq(&first, &second));
    let diagnostics = observer.snapshot();
    assert_eq!(diagnostics.work["discovery.requests"], 2);
    assert_eq!(diagnostics.work["discovery.roots"], 1);
    assert_eq!(diagnostics.work["discovery.cache_hits"], 1);
}

#[test]
fn disabled_session_allocates_no_work_ledger() {
    let session = AnalysisSession::disabled();
    let source = fixture_root().join("consumer.mts");
    session.read_source(&source).unwrap();
    assert_eq!(session.work_snapshot(), SessionWorkSnapshot::default());
}

#[test]
fn canonical_manifests_memoize_successes_and_parse_failures() {
    let observer = InvocationObserver::new(true);
    let session = AnalysisSession::new(Some(Arc::clone(&observer)));
    let valid_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/playwright-config-path-graph/fixture");
    let config_path = valid_root.join("custom.no-mistakes.yml");
    let tsconfig_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/napi/analyze-project-dynamic-import-reachability/tsconfig.json");
    let first_config = session.config(&valid_root, Some(&config_path)).unwrap();
    let second_config = session.config(&valid_root, Some(&config_path)).unwrap();
    let first_tsconfig = session.tsconfig(&valid_root, Some(&tsconfig_path)).unwrap();
    let second_tsconfig = session.tsconfig(&valid_root, Some(&tsconfig_path)).unwrap();
    assert!(Arc::ptr_eq(&first_config, &second_config));
    assert!(Arc::ptr_eq(&first_tsconfig, &second_tsconfig));

    let invalid_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/impacted-checks/multi-framework");
    let invalid_config = invalid_root.join("invalid.no-mistakes.yml");
    let first_error = session
        .config(&invalid_root, Some(Path::new("invalid.no-mistakes.yml")))
        .unwrap_err();
    let second_error = session
        .config(&invalid_root, Some(&invalid_config))
        .unwrap_err();
    assert_eq!(first_error.to_string(), second_error.to_string());

    let work = observer.snapshot().work;
    assert_eq!(work["manifest.requests"], 6);
    assert_eq!(work["manifest.parses"], 3);
    assert_eq!(work["manifest.cache_hits"], 3);
    assert_eq!(work["manifest.errors"], 1);
    let keyed = session.work_snapshot().source_reads;
    assert_eq!(keyed[&normalize_path(&config_path)], 1);
    assert_eq!(keyed[&normalize_path(&tsconfig_path)], 1);
    assert_eq!(keyed[&normalize_path(&invalid_config)], 1);
}

#[test]
fn recovered_parser_counts_physical_work_once_per_key() {
    let observer = InvocationObserver::new(true);
    let session = AnalysisSession::new(Some(Arc::clone(&observer)));
    let path = normalize_path(&fixture_root().join("consumer.mts"));
    let source = session.read_source(&path).unwrap();

    crate::ast::with_request_parse_cache(|| {
        for _ in 0..2 {
            session
                .with_recovered_program(&path, &source, |program, _, error| {
                    assert!(error.is_none());
                    program.body.len()
                })
                .unwrap();
        }
    });

    let snapshot = session.work_snapshot();
    assert_eq!(snapshot.parse_attempts[&path], 1);
    let work = observer.snapshot().work;
    assert_eq!(work["parse.requests"], 2);
    assert_eq!(work["parse.files"], 1);
    assert!(!work.contains_key("parse.errors"));
}

#[test]
fn strict_parser_memoizes_fixture_parse_failure_and_counts_physical_work_once() {
    let observer = InvocationObserver::new(true);
    let session = AnalysisSession::new(Some(Arc::clone(&observer)));
    let path =
        normalize_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-cases/integration-tests/parse-errors/fixture/vitest.syntax-error.mts",
        ));
    let source = session.read_source(&path).unwrap();

    crate::ast::with_request_parse_cache(|| {
        for _ in 0..2 {
            assert!(session.with_program(&path, &source, |_, _| ()).is_err());
        }
    });

    let snapshot = session.work_snapshot();
    assert_eq!(snapshot.parse_attempts[&path], 1);
    let work = observer.snapshot().work;
    assert_eq!(work["parse.requests"], 2);
    assert_eq!(work["parse.files"], 1);
    assert_eq!(work["parse.errors"], 1);
}

#[test]
fn legacy_symbol_parse_mode_is_distinct_from_standard_javascript() {
    let observer = InvocationObserver::new(true);
    let session = AnalysisSession::new(Some(Arc::clone(&observer)));
    let path = normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/symbols-output/fixture/src/types-in-js.js"),
    );
    let source = session.read_source(&path).unwrap();

    crate::ast::with_request_parse_cache(|| {
        let standard_diagnostic = session
            .with_recovered_program(&path, &source, |_, _, diagnostic| diagnostic)
            .unwrap();
        let legacy_diagnostic = session
            .with_legacy_symbols_program(&path, &source, |_, _, diagnostic| diagnostic)
            .unwrap();
        assert!(standard_diagnostic.is_some());
        assert!(legacy_diagnostic.is_none());
    });

    assert_eq!(session.work_snapshot().parse_attempts[&path], 2);
    let work = observer.snapshot().work;
    assert_eq!(work["parse.requests"], 2);
    assert_eq!(work["parse.files"], 2);
    assert_eq!(work["parse.errors"], 1);
}

#[test]
fn legacy_symbol_parse_mode_memoizes_recovered_diagnostic() {
    let observer = InvocationObserver::new(true);
    let session = AnalysisSession::new(Some(Arc::clone(&observer)));
    let path = normalize_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/symbols-output/fixture/src/recoverable-diagnostic.mts",
    ));
    let source = session.read_source(&path).unwrap();

    crate::ast::with_request_parse_cache(|| {
        for _ in 0..2 {
            let (symbols, diagnostic) = session
                .with_legacy_symbols_program(&path, &source, |program, source, diagnostic| {
                    (
                        crate::codebase::ts_symbols::extract_symbols_from_program(program, source),
                        diagnostic,
                    )
                })
                .unwrap();
            assert!(diagnostic.is_some());
            assert_eq!(
                symbols
                    .exports
                    .iter()
                    .map(|export| export.name.as_str())
                    .collect::<Vec<_>>(),
                vec!["x", "recovered"]
            );
        }
    });

    assert_eq!(session.work_snapshot().parse_attempts[&path], 1);
    let work = observer.snapshot().work;
    assert_eq!(work["parse.requests"], 2);
    assert_eq!(work["parse.files"], 1);
    assert_eq!(work["parse.errors"], 1);
}
