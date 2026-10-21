use super::run_all;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

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
fn supplemental_call_sources_collect_facts_once() {
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
    assert_eq!(
        work.get("parse.files_after_extract"),
        work.get("parse.files")
    );
    assert!(results
        .rules
        .iter()
        .any(|finding| { finding.rule == no_mistakes::codebase::rules::FINITE_SET_CONSISTENCY }));

    // The alternate saved config makes the skipped call source supplemental.
    // That config builds no graph. fact_collections ignores the call-only pass,
    // so the supplemental path's own parse attempt is the once-only proof.
    let observer = no_mistakes::diagnostics::InvocationObserver::new(true);
    let session =
        no_mistakes::codebase::analysis_session::AnalysisSession::new(Some(Arc::clone(&observer)));
    let supplemental_root = fixture.path().canonicalize().unwrap();
    let result = {
        let _guard = no_mistakes::diagnostics::InvocationGuard::install(Arc::clone(&observer));
        super::super::run_all::run_all_with_session(
            &session,
            supplemental_root.clone(),
            Some(fixture.path().join(".no-mistakes-supplemental.yml")),
            None,
            false,
            false,
            false,
        )
        .unwrap()
    };
    let work = observer.snapshot().work;
    let call_source = no_mistakes::codebase::ts_resolver::normalize_path(
        &supplemental_root.join("generated/schedules.test.mts"),
    );
    let parse_attempts = session.work_snapshot().parse_attempts;
    assert!(
        parse_attempts
            .get(&call_source)
            .is_some_and(|count| *count == 1),
        "{parse_attempts:#?}"
    );
    assert_eq!(work.get("check.fact_collections"), Some(&1), "{work:#?}");
    assert_eq!(work.get("check.prepare.catalog"), Some(&1), "{work:#?}");
    assert_eq!(work.get("check.prepare.independent"), Some(&1), "{work:#?}");
    assert!(!work.contains_key("graph.builds"), "{work:#?}");
    assert_eq!(
        work.get("parse.files_after_extract"),
        work.get("parse.files")
    );
    assert!(result
        .rules
        .iter()
        .any(|finding| { finding.rule == no_mistakes::codebase::rules::FINITE_SET_CONSISTENCY }));
}

#[test]
fn prepare_join_serial_time_does_not_exceed_its_wall_time() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/finite-set-consistency/call-literals/supplemental-skipped");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let observer = no_mistakes::diagnostics::InvocationObserver::with_progress(true, true);
    let results = {
        let _guard = no_mistakes::diagnostics::InvocationGuard::install(Arc::clone(&observer));
        run_all(root, None, None).unwrap()
    };
    assert!(results
        .rules
        .iter()
        .any(|finding| { finding.rule == no_mistakes::codebase::rules::FINITE_SET_CONSISTENCY }));

    let snapshot = observer.snapshot();
    let wall = snapshot
        .timings
        .iter()
        .find(|timing| {
            timing.label == "check.prepare"
                && timing.kind == no_mistakes::diagnostics::TimingKind::Serial
        })
        .expect("check.prepare records the join wall")
        .duration;
    let serial = snapshot
        .timings
        .iter()
        .filter(|timing| {
            timing.label.starts_with("check.prepare")
                && timing.kind == no_mistakes::diagnostics::TimingKind::Serial
        })
        .map(|timing| timing.duration)
        .sum::<Duration>();
    assert!(
        serial <= wall,
        "prepare serial {serial:?} exceeded wall {wall:?}: {:#?}",
        snapshot.timings
    );
    for label in ["check.prepare.catalog", "check.prepare.independent"] {
        let timing = snapshot
            .timings
            .iter()
            .find(|timing| timing.label == label)
            .unwrap_or_else(|| panic!("missing {label} in {:#?}", snapshot.timings));
        assert_eq!(
            timing.kind,
            no_mistakes::diagnostics::TimingKind::Parallel,
            "{label} must not add into the prepare wall"
        );
        assert!(
            timing.duration <= wall,
            "{label} {:?} exceeded wall {wall:?}",
            timing.duration
        );
    }
    assert_eq!(snapshot.work.get("check.prepare.catalog"), Some(&1));
    assert_eq!(snapshot.work.get("check.prepare.independent"), Some(&1));
}
