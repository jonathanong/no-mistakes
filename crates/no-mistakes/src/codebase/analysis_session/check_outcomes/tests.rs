use super::*;

#[test]
fn failed_saved_source_only_invalidates_consumers_that_need_it() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check-consumer-isolation");
    let broken = root.join("broken.ts");
    let clean = root.join("clean.ts");
    let session = AnalysisSession::disabled();
    session.enable_partial_checks();
    session.register_check_demand("needs-broken", [broken.clone()]);
    session.register_check_demand("needs-clean", [clean]);
    let source = session.read_source(&broken).unwrap();
    assert!(source.contains("export const broken"));
    session.record_check_source_failure(&broken, "failed to parse source");
    session.record_check_source_failure(&broken, "failed to parse source");
    assert!(!session.check_consumer_ready("needs-broken"));
    assert!(session.check_consumer_ready("needs-clean"));
    let failures = session.check_failures();
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].file.as_ref(), Some(&broken));
    assert_eq!(failures[0].consumer, "needs-broken");
    assert_eq!(
        serde_json::to_value(&failures[0]).unwrap().get("property"),
        None
    );
}

#[test]
fn standalone_errors_remain_errors_and_partial_siblings_survive() {
    let session = AnalysisSession::disabled();
    assert!(session
        .capture_check_result::<()>("broken", Err(anyhow::anyhow!("bad options")))
        .is_err());
    session.enable_partial_checks();
    assert_eq!(
        session
            .capture_check_result::<()>("broken", Err(anyhow::anyhow!("bad options")))
            .unwrap(),
        None
    );
    assert_eq!(
        session.capture_check_result("healthy", Ok(7)).unwrap(),
        Some(7)
    );
    assert_eq!(
        session.check_failures(),
        vec![CheckFailure::new("broken", "bad options")]
    );
}

#[test]
fn scoped_ledgers_share_resources_without_failure_leakage() {
    let session = AnalysisSession::disabled();
    session.enable_partial_checks();
    session.record_check_failure(CheckFailure::new("rule", "failed in first scope"));
    let scoped = session.scoped_checks();
    assert!(std::sync::Arc::ptr_eq(
        &session.resources,
        &scoped.resources
    ));
    assert!(scoped.partial_checks_enabled());
    assert!(scoped.check_failures().is_empty());
    assert!(scoped.check_consumer_ready("rule"));
    scoped.record_check_failure(CheckFailure::new("other", "second scope"));
    assert_eq!(session.check_failures().len(), 1);
    assert!(session.check_consumer_ready("other"));
}

#[test]
fn late_demands_replay_known_source_failure() {
    let session = AnalysisSession::disabled();
    let file = PathBuf::from("/repo/broken.ts");
    session.record_check_source_failure(&file, "parse failed");
    session.register_check_demand("late", [file.clone()]);
    assert_eq!(session.check_failures()[0].file, Some(file));
}

#[test]
fn typed_source_failure_retains_structured_location() {
    let session = AnalysisSession::disabled();
    session.enable_partial_checks();
    let file = PathBuf::from("/repo/broken.ts");
    let error =
        crate::codebase::check_outcome::CheckInputFailure::source(file.clone(), "parse failed");
    session
        .capture_check_result::<()>("consumer", Err(error.into()))
        .unwrap();
    assert_eq!(session.check_failures()[0].file, Some(file));
}
