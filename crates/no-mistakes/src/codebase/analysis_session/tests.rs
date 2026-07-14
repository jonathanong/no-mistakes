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
fn typed_documents_memoize_successes_and_parse_failures() {
    let observer = InvocationObserver::new(true);
    let session = AnalysisSession::new(Some(Arc::clone(&observer)));
    let valid = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/ts-resolver/fixture/explicit-json/src/data.json");
    let invalid = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/invalid-tsconfig/fixture/tsconfig.json");

    let first = session
        .parse_document("package", &valid, |source| {
            serde_json::from_str::<serde_json::Value>(source).map_err(Into::into)
        })
        .unwrap();
    let second = session
        .parse_document("package", &valid, |_| panic!("cached document reparsed"))
        .unwrap();
    assert!(Arc::ptr_eq(&first, &second));
    let first_error = session
        .parse_document("tsconfig", &invalid, |source| {
            serde_json::from_str::<serde_json::Value>(source).map_err(Into::into)
        })
        .unwrap_err();
    let second_error = session
        .parse_document::<serde_json::Value>("tsconfig", &invalid, |_| {
            panic!("cached invalid document reparsed")
        })
        .unwrap_err();
    assert_eq!(first_error, second_error);

    let work = observer.snapshot().work;
    assert_eq!(work["manifest.requests"], 4);
    assert_eq!(work["manifest.parses"], 2);
    assert_eq!(work["manifest.cache_hits"], 2);
    assert_eq!(work["manifest.errors"], 1);
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
