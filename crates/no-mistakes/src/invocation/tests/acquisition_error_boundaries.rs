use super::*;
use std::fs::File;
use std::time::Duration;

#[test]
fn cli_path_resolution_error_releases_early_invocation_deadline() {
    let _serial = deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let result = InvocationGuard::acquire_resolving_path_with_json(
        InvocationOptions {
            invocation_timeout: Some(Duration::from_secs(5)),
            ..InvocationOptions::default()
        },
        true,
        || Err(anyhow::anyhow!("fixture path resolver failed")),
        true,
    );
    let error = result.err().expect("resolver failure must be returned");
    assert!(error.to_string().contains("fixture path resolver failed"));
    assert!(active_deadline().read().unwrap().is_none());
}

#[test]
fn lock_classifier_preserves_successful_open_result() {
    let file = File::open(fixture_path("lock-holder/invocation.lock")).unwrap();
    let acquired = super::super::acquisition::classify_lock_result(Ok(file), None, false).unwrap();
    assert!(acquired.metadata().unwrap().is_file());
}
