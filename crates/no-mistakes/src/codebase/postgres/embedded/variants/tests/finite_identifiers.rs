use super::*;

#[test]
fn exact_vitest_iterator_bounds_only_unescaped_own_sql_fields() {
    let path = crate::test_support::rule_fixture_root("postgres-no-test-state-mutation")
        .join("tests/finite-identifiers.test.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = extract_embedded_sql_from_source(
        &path,
        &source,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert_eq!(facts.calls.len(), 9);
    assert_eq!(
        super::texts(&facts.calls[0]),
        [
            "/* finite */ UPDATE records SET first_id = $2 WHERE id = $1",
            "/* finite */ UPDATE records SET second_id = $2 WHERE id = $1",
            "/* finite */ UPDATE records SET third_id = $2 WHERE id = $1",
        ]
    );
    assert_eq!(
        super::texts(&facts.calls[1]),
        [
            "/* finite_schema */ SELECT 1",
            "/* finite_schema */ DROP TABLE records",
        ]
    );
    for call in &facts.calls[2..] {
        assert!(call.variants.is_empty(), "unknown runtime domain: {call:?}");
    }
}

#[test]
fn malformed_iterator_maps_and_callback_effects_keep_runtime_sql_unknown() {
    let path = crate::test_support::rule_fixture_root("postgres-no-test-state-mutation")
        .join("tests/finite-iterator-shapes.test.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = extract_embedded_sql_from_source(
        &path,
        &source,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert_eq!(facts.calls.len(), 23);
    for call in &facts.calls {
        assert!(
            call.variants.is_empty(),
            "runtime key or map effect: {call:?}"
        );
    }
}

#[test]
fn unbound_iterator_member_call_does_not_prove_a_finite_domain() {
    let path = crate::test_support::rule_fixture_root("postgres-no-test-state-mutation")
        .join("tests/finite-iterator-member-callee.test.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = extract_embedded_sql_from_source(
        &path,
        &source,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert_eq!(facts.calls.len(), 1);
    assert!(facts.calls[0].variants.is_empty());
}

#[test]
fn optional_iterator_call_does_not_prove_a_finite_domain() {
    let path = crate::test_support::rule_fixture_root("postgres-no-test-state-mutation")
        .join("tests/finite-iterator-optional.test.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = extract_embedded_sql_from_source(
        &path,
        &source,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert_eq!(facts.calls.len(), 1);
    assert!(facts.calls[0].variants.is_empty());
}

#[test]
fn value_binding_read_preserves_a_finite_iterator_domain() {
    let path = crate::test_support::rule_fixture_root("postgres-no-test-state-mutation")
        .join("tests/finite-iterator-value-read.test.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = extract_embedded_sql_from_source(
        &path,
        &source,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert_eq!(facts.calls.len(), 1);
    assert_eq!(
        super::texts(&facts.calls[0]),
        [
            "/* valueRead */ SELECT 1 FROM safe_one",
            "/* valueRead */ SELECT 1 FROM safe_two"
        ]
    );
}

#[test]
fn callback_parameter_and_body_shapes_do_not_prove_runtime_keys() {
    let path = crate::test_support::rule_fixture_root("postgres-no-test-state-mutation")
        .join("tests/finite-iterator-callback-shapes.test.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = extract_embedded_sql_from_source(
        &path,
        &source,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert_eq!(facts.calls.len(), 5);
    for call in &facts.calls {
        assert!(call.variants.is_empty(), "callback shape: {call:?}");
    }
}

#[test]
fn oversized_iterator_callback_stops_before_proving_runtime_sql() {
    let path = crate::test_support::rule_fixture_root("postgres-no-test-state-mutation")
        .join("tests/finite-iterator-budget.test.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = extract_embedded_sql_from_source(
        &path,
        &source,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert_eq!(facts.calls.len(), 1);
    assert!(facts.calls[0].variants.is_empty());
}

#[test]
fn escaped_or_replaced_vitest_iterator_has_no_finite_domain() {
    for name in [
        "finite-iterator-escape.test.ts",
        "finite-iterator-write.test.ts",
        "finite-iterator-eval.test.ts",
    ] {
        let path = crate::test_support::rule_fixture_root("postgres-no-test-state-mutation")
            .join("tests")
            .join(name);
        let source = std::fs::read_to_string(&path).unwrap();
        let facts = extract_embedded_sql_from_source(
            &path,
            &source,
            &EmbeddedSqlOptions::configured("@example/db", &[]),
        );
        assert_eq!(facts.calls.len(), 1, "{name}");
        assert!(facts.calls[0].variants.is_empty(), "{name}");
    }
}

#[test]
fn selected_records_cannot_escape_as_method_or_tag_receivers() {
    let path = crate::test_support::rule_fixture_root("postgres-no-test-state-mutation")
        .join("tests/finite-iterator-receiver.test.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = extract_embedded_sql_from_source(
        &path,
        &source,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert_eq!(facts.calls.len(), 3);
    assert!(
        facts.calls[0].variants.is_empty(),
        "method receiver escaped"
    );
    assert!(facts.calls[1].variants.is_empty(), "tag receiver escaped");
    assert_eq!(super::texts(&facts.calls[2]), ["/* recordBind */ SELECT 1"]);
}

#[test]
fn for_loop_assignment_targets_cannot_replace_selected_sql() {
    let path = crate::test_support::rule_fixture_root("postgres-no-test-state-mutation")
        .join("tests/finite-iterator-targets.test.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = extract_embedded_sql_from_source(
        &path,
        &source,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert_eq!(facts.calls.len(), 3);
    assert!(
        facts.calls[0].variants.is_empty(),
        "for-of target wrote SQL"
    );
    assert!(
        facts.calls[1].variants.is_empty(),
        "for-in target wrote SQL"
    );
    assert_eq!(
        super::texts(&facts.calls[2]),
        ["/* independentIteration */ SELECT 1"]
    );
}
