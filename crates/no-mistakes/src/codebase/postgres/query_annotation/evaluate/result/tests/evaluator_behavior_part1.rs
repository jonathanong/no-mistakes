#[test]
fn arithmetic_and_primitive_comparisons_keep_exact_values() {
    let values = values("arithmetic.mts");
    let mut mismatches = Vec::new();
    let mut check = |marker: &str, matches: bool| {
        if !matches {
            mismatches.push(format!("{marker}: {}", describe(event(&values, marker))));
        }
    };
    check(
        "numeric-add",
        matches!(event(&values, "numeric-add"), Value::Number(3)),
    );
    for (marker, expected) in [
        ("numeric-strict-equal", true),
        ("numeric-strict-unequal", false),
        ("numeric-equal", true),
        ("numeric-unequal", true),
        ("numeric-strict-not-equal", false),
        ("numeric-greater", true),
        ("numeric-not-greater", false),
        ("numeric-less", true),
        ("numeric-not-less", false),
        ("numeric-greater-equal", true),
        ("numeric-not-greater-equal", false),
        ("numeric-less-equal", true),
        ("numeric-not-less-equal", false),
    ] {
        check(
            marker,
            matches!(event(&values, marker), Value::Boolean(value) if *value == expected),
        );
    }
    for marker in ["text-add", "mixed-add"] {
        check(
            marker,
            matches!(event(&values, marker), Value::Prefix(text, true, None) if text == "SELECT 1"),
        );
    }
    for marker in [
        "numeric-overflow",
        "numeric-subtract-opaque",
        "numeric-multiply-opaque",
        "numeric-divide-opaque",
        "text-comparison-opaque",
        "strict-record-comparison",
        "unary-zero",
        "unary-negative-zero",
        "fractional-primitive",
        "unsafe-integer-primitive",
    ] {
        check(marker, matches!(event(&values, marker), Value::Unknown));
    }
    for marker in [
        "record-coercion",
        "reversed-record-coercion",
        "array-coercion",
    ] {
        check(marker, matches!(event(&values, marker), Value::Failure(_)));
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
    assert_eq!(values.len(), 29);
}

#[test]
fn branches_select_truthy_numeric_paths_and_preserve_early_returns() {
    let values = values("control.mts");
    for (marker, expected) in [
        ("branch-true", "SELECT 1"),
        ("branch-false", "SELECT 2"),
        ("branch-zero", "SELECT 2"),
        ("branch-nonzero", "SELECT 1"),
        ("branch-comparison", "SELECT 1"),
        ("branch-reversed", "SELECT 2"),
        ("branch-early-return", "SELECT 1"),
        ("branch-same", "SELECT 3"),
    ] {
        assert!(
            matches!(event(&values, marker), Value::Prefix(text, true, None) if text == expected),
            "{marker}: {}",
            describe(event(&values, marker))
        );
    }
    for marker in [
        "branch-local-binding",
        "branch-opaque-condition",
        "branch-failed-condition",
    ] {
        assert!(
            matches!(event(&values, marker), Value::Failure(_)),
            "{marker}: {}",
            describe(event(&values, marker))
        );
    }
    let value = event(&values, "branch-different");
    assert!(matches!(value, Value::Joined(_)), "{}", describe(value));
    assert_eq!(values.len(), 12);
}

#[test]
fn array_composition_executes_callbacks_and_rejects_uncertain_shapes() {
    let values = values("collections.mts");
    for (marker, expected) in [
        ("collection-join", "SELECT 1"),
        ("collection-default-separator", "SELECT,1"),
        ("collection-push", "SELECT 2"),
        ("collection-map", "SELECT 1 UNION ALL SELECT 2"),
        ("collection-for-each", "SELECT 3"),
    ] {
        assert!(
            matches!(event(&values, marker), Value::Prefix(text, true, None) if text == expected),
            "{marker}: {}",
            describe(event(&values, marker))
        );
    }
    for marker in [
        "collection-bad-separator",
        "collection-object",
        "collection-hole",
        "collection-deleted",
        "collection-conditional",
        "collection-missing-callback",
    ] {
        assert!(
            matches!(event(&values, marker), Value::Failure(_)),
            "{marker}: {}",
            describe(event(&values, marker))
        );
    }
}
