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

#[test]
fn postgres_rules_share_one_prepared_ts_parse() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/conflict-ordering/shared-prepared");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let observer = no_mistakes::diagnostics::InvocationObserver::new(true);

    let result = {
        let _guard = no_mistakes::diagnostics::InvocationGuard::install(Arc::clone(&observer));
        run_all(root, None, None).unwrap()
    };
    let work = observer.snapshot().work;

    assert!(result.rules.is_empty(), "{:#?}", result.rules);
    assert_eq!(work.get("parse.files"), Some(&1), "{work:#?}");
    assert_eq!(
        work.get("parse.files_after_extract"),
        Some(&1),
        "rules must not parse again after the shared extract: {work:#?}"
    );
}

#[test]
fn postgres_offset_rule_reuses_the_requested_prepared_projection() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/no-offset/legacy-single-pass");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let observer = no_mistakes::diagnostics::InvocationObserver::new(true);

    let result = {
        let _guard = no_mistakes::diagnostics::InvocationGuard::install(Arc::clone(&observer));
        run_all(root, None, None).unwrap()
    };
    let work = observer.snapshot().work;

    assert!(result.rules.is_empty(), "{:#?}", result.rules);
    assert_eq!(work.get("parse.files"), Some(&1), "{work:#?}");
    assert_eq!(
        work.get("parse.files_after_extract"),
        Some(&1),
        "the prepared SQL projection parses its host exactly once: {work:#?}"
    );
}

#[test]
fn supplemental_call_sources_share_one_fact_collection_and_graph() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/finite-set-consistency/call-literals/supplemental-skipped");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let observer = no_mistakes::diagnostics::InvocationObserver::new(true);
    let results = {
        let _guard = no_mistakes::diagnostics::InvocationGuard::install(Arc::clone(&observer));
        run_all(root, None, None).unwrap()
    };
    let work = observer.snapshot().work;
    assert_eq!(work.get("check.fact_collections"), Some(&1), "{work:#?}");
    assert_eq!(work.get("check.prepare.catalog"), Some(&1), "{work:#?}");
    assert_eq!(work.get("check.prepare.independent"), Some(&1), "{work:#?}");
    assert_eq!(work.get("graph.builds"), Some(&1), "{work:#?}");
    assert_eq!(work.get("parse.files_after_extract"), work.get("parse.files"));
    assert!(results
        .rules
        .iter()
        .any(|finding| { finding.rule == no_mistakes::codebase::rules::FINITE_SET_CONSISTENCY }));

    // The alternate saved config makes the skipped call source supplemental.
    let first_observer = Arc::clone(&observer);
    let observer = no_mistakes::diagnostics::InvocationObserver::new(true);
    let result = {
        let _guard = no_mistakes::diagnostics::InvocationGuard::install(Arc::clone(&observer));
        run_all(
            fixture.path().canonicalize().unwrap(),
            Some(fixture.path().join(".no-mistakes-supplemental.yml")),
            None,
        )
        .unwrap()
    };
    let work = observer.snapshot().work;
    assert_eq!(work.get("check.fact_collections"), Some(&1), "{work:#?}");
    assert_eq!(work.get("check.prepare.catalog"), Some(&1), "{work:#?}");
    assert_eq!(work.get("check.prepare.independent"), Some(&1), "{work:#?}");
    assert!(work.get("graph.builds").is_none(), "{work:#?}");
    assert_eq!(work.get("parse.files_after_extract"), work.get("parse.files"));
    assert_eq!(
        first_observer.snapshot().work.get("check.prepare.catalog"),
        Some(&1)
    );
    assert!(result
        .rules
        .iter()
        .any(|finding| { finding.rule == no_mistakes::codebase::rules::FINITE_SET_CONSISTENCY }));
}
