use super::*;
#[test]
fn polling_controls_are_strict_and_unknown_live_ids_return_null() {
    assert_eq!(
        diagnostics_snapshot_impl(serde_json::json!({"id":"unknown-control-test"})).unwrap(),
        "null"
    );
    for options in [
        serde_json::json!({}),
        serde_json::json!({"id":123}),
        serde_json::json!({"id":"unknown-control-test","extra":true}),
    ] {
        assert!(diagnostics_snapshot_impl(options).is_err());
    }
}

#[test]
fn serialization_errors_preserve_the_native_failure_reason() {
    struct Unsupported;
    impl serde::Serialize for Unsupported {
        fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("unsupported diagnostic value"))
        }
    }
    let error = serde_json::to_string(&Unsupported)
        .map_err(super::super::async_task::to_napi_error)
        .unwrap_err();
    assert_eq!(error.reason, "unsupported diagnostic value");
}
