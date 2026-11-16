#[test]
fn trusted_tag_binds_raw_coercion_and_executor_effects_keep_distinct_outcomes() {
    let values = values("tags.mts");
    for (marker, expected) in [
        ("tag-scalar-bind", "SELECT $1"),
        ("tag-record-bind", "SELECT $1"),
        ("raw-number", "SELECT 2"),
        ("raw-source-text", "SELECT\\n3"),
        ("raw-boolean", "SELECT true"),
    ] {
        let Value::Prefix(text, true, _) = event(&values, marker) else {
            panic!("{marker}: {}", describe(event(&values, marker)))
        };
        assert_eq!(
            crate::codebase::postgres::embedded::publish_placeholders_with_positions(text.clone())
                .0,
            expected,
            "{marker}"
        );
    }
    for (marker, expected) in [
        ("tag-leading-builder", ""),
        ("tag-trailing-builder", "SELECT "),
    ] {
        assert!(
            matches!(event(&values, marker), Value::Prefix(text, false, _) if text == expected),
            "{marker}: {}",
            describe(event(&values, marker))
        );
    }
    for marker in [
        "tag-call-effect",
        "raw-record-coercion",
        "tag-shadowed-binding",
        "raw-shadowed-binding",
        "executor-later-getter",
    ] {
        assert!(
            matches!(event(&values, marker), Value::Failure(_)),
            "{marker}: {}",
            describe(event(&values, marker))
        );
    }
    assert!(
        values.iter().any(
            |(_, value)| matches!(value, Value::Prefix(text, true, None) if text == "SELECT 8")
        ),
        "opaque tag must still invoke its callback"
    );
}
