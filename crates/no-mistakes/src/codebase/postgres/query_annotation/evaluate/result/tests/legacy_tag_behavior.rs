use super::{event, outputs_mode, Value};

#[test]
fn legacy_string_tags_require_the_original_function_identity() {
    let values = outputs_mode("coverage-evaluator", "legacy-sync.mts", true);
    for (marker, expected) in [
        ("legacy-direct", "SELECT 1"),
        ("legacy-captured", "SELECT 2"),
    ] {
        assert!(
            matches!(event(&values, marker), Value::Prefix(text, true, Some(_)) if text == expected),
            "{marker}"
        );
    }
    let shadowed: Vec<_> = values
        .iter()
        .filter(|(marker, _)| marker.contains("legacy-shadowed"))
        .map(|(_, value)| value)
        .collect();
    assert_eq!(shadowed.len(), 2);
    assert!(matches!(shadowed[0], Value::Prefix(text, true, Some(_)) if text == "SELECT 3"));
    assert!(matches!(shadowed[1], Value::Unknown));
    assert_eq!(values.len(), 4);
}

#[test]
fn async_legacy_tags_keep_a_promise_until_awaited() {
    let values = outputs_mode("coverage-evaluator", "legacy-async.mts", true);
    let Value::Promise(value) = event(&values, "legacy-async-promise") else {
        panic!("async tags must retain their promise result")
    };
    assert!(matches!(value.as_ref(), Value::Prefix(text, true, Some(_)) if text == "SELECT 4"));
    assert!(
        matches!(event(&values, "legacy-async-awaited"), Value::Prefix(text, true, Some(_)) if text == "SELECT 5")
    );
    assert_eq!(values.len(), 2);
}

#[test]
fn a_static_return_function_is_not_the_legacy_tag_implementation_shape() {
    let values = outputs_mode("coverage-evaluator", "legacy-static.mts", true);
    assert!(matches!(
        event(&values, "legacy-static-return-untrusted"),
        Value::Unknown
    ));
    assert_eq!(values.len(), 1);
}
