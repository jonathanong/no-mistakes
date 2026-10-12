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
