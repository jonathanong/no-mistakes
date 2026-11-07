use super::*;

#[test]
fn live_registry_is_request_scoped_rejects_duplicates_and_removes_completed_requests() {
    assert!(
        DiagnosticRequestGuard::extract(&mut serde_json::json!({"root":"."}))
            .unwrap()
            .is_none()
    );
    assert!(super::super::current().is_none());
    let mut options = serde_json::json!({"diagnosticsId":"live-registry-test", "root":"."});
    let guard = DiagnosticRequestGuard::extract(&mut options)
        .unwrap()
        .unwrap();
    assert_eq!(options, serde_json::json!({"root":"."}));
    assert!(progress_for_request("unrelated-id").is_none());
    let snapshot = std::thread::spawn(|| progress_for_request("live-registry-test"))
        .join()
        .unwrap()
        .unwrap();
    assert!(snapshot.active.is_empty());
    assert!(DiagnosticRequestGuard::extract(
        &mut serde_json::json!({"diagnosticsId":"live-registry-test"})
    )
    .is_err());
    guard
        .observer
        .trace("phase", super::super::TimingKind::Serial, || {
            assert_eq!(
                progress_for_request("live-registry-test").unwrap().active[0].phase,
                ["phase"]
            );
        });
    assert_eq!(
        progress_for_request("live-registry-test")
            .unwrap()
            .completed_spans,
        1
    );
    drop(guard);
    assert!(super::super::current().is_none());
    assert!(progress_for_request("live-registry-test").is_none());
    assert!(!requests()
        .lock()
        .unwrap()
        .contains_key("live-registry-test"));
    let _reuse = DiagnosticRequestGuard::extract(
        &mut serde_json::json!({"diagnosticsId":"live-registry-test"}),
    )
    .unwrap();
}

#[test]
fn invalid_ids_are_rejected_without_installing_an_observer() {
    for value in [
        serde_json::json!(null),
        serde_json::json!(true),
        serde_json::json!(2),
        serde_json::json!(""),
    ] {
        let error =
            DiagnosticRequestGuard::extract(&mut serde_json::json!({"diagnosticsId":value}))
                .err()
                .unwrap();
        assert!(error.to_string().contains("nonempty string"));
        assert!(super::super::current().is_none());
    }
}

#[test]
fn structured_collection_is_opt_in_and_does_not_register_completed_payloads() {
    let mut disabled = serde_json::json!({"__collectDiagnostics":false,"root":"."});
    assert!(DiagnosticRequestGuard::extract(&mut disabled)
        .unwrap()
        .is_none());
    assert_eq!(disabled, serde_json::json!({"root":"."}));
    let mut enabled = serde_json::json!({"__collectDiagnostics":true,"root":"."});
    let guard = DiagnosticRequestGuard::extract(&mut enabled)
        .unwrap()
        .unwrap();
    assert_eq!(enabled, serde_json::json!({"root":"."}));
    assert!(guard.id.is_none());
    assert!(!guard.observer.progress_enabled());
    assert!(guard.report(true).is_some());
    drop(guard);
    assert!(super::super::current().is_none());
    for value in [
        serde_json::json!(null),
        serde_json::json!(1),
        serde_json::json!("yes"),
    ] {
        assert!(DiagnosticRequestGuard::extract(
            &mut serde_json::json!({"__collectDiagnostics":value})
        )
        .is_err());
        assert!(super::super::current().is_none());
    }
}

#[test]
fn verbose_collection_can_share_a_live_progress_id_without_retaining_it() {
    let mut options =
        serde_json::json!({"__collectDiagnostics":true,"diagnosticsId":"verbose-progress-test"});
    let guard = DiagnosticRequestGuard::extract(&mut options)
        .unwrap()
        .unwrap();
    assert!(guard.observer.verbose());
    assert!(progress_for_request("verbose-progress-test").is_some());
    guard
        .observer
        .trace("phase", super::super::TimingKind::Serial, || ());
    assert!(!guard.report(true).unwrap().timings.is_empty());
    drop(guard);
    assert!(progress_for_request("verbose-progress-test").is_none());
}
