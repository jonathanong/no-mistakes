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
