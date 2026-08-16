use super::run_all;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn aggregate_check_releases_cached_asts_after_extract() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/playwright/html-id-rule-composition/html-disabled-unique");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let observer = no_mistakes::diagnostics::InvocationObserver::new(true);

    no_mistakes::ast::with_request_parse_cache(|| {
        let _guard = no_mistakes::diagnostics::InvocationGuard::install(Arc::clone(&observer));
        run_all(root, None, None).unwrap();
        assert_eq!(
            no_mistakes::ast::request_parse_cache_len(),
            0,
            "check must drop request-scoped OXC programs after extract"
        );
    });

    let work = observer.snapshot().work;
    let parse_files = *work.get("parse.files").expect("extract must parse files");
    assert!(parse_files > 0, "{work:#?}");
    assert_eq!(
        work.get("parse.files_after_extract").copied(),
        Some(parse_files),
        "domain checks must not parse after extract: {work:#?}"
    );
}
