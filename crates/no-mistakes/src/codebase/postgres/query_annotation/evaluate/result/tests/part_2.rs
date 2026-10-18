#[test]
fn opaque_consumers_revisit_mutated_argument_slots_without_repeating_installers() {
    let values = outputs("helper-tracing-callback-installers");
    for (marker, text) in [
        ("object-first", "SELECT 1"),
        ("installer-first", "SELECT 2"),
        ("cyclic-container", "SELECT 3"),
        ("duplicate-installed-callback", "SELECT 4"),
        ("duplicate-installer-body", "SELECT 5"),
    ] {
        assert!(
            matches!(event(&values, marker), Value::Prefix(sql, true, _) if sql == text),
            "{marker}"
        );
    }
    assert_eq!(
        values
            .iter()
            .filter(|(sql, _)| sql.contains("separate-consumers"))
            .count(),
        2
    );
    assert_eq!(values.len(), 7);
}

#[test]
fn numeric_unary_operators_preserve_coercion_effects() {
    let values = outputs("helper-tracing-discarded-values");
    for marker in [
        "unary plus coercion",
        "unary negation coercion",
        "bitwise not coercion",
    ] {
        assert!(matches!(event(&values, marker), Value::Unknown), "{marker}");
    }
    for marker in [
        "typeof is noncoercive",
        "logical not is noncoercive",
        "void callback",
    ] {
        assert!(
            matches!(event(&values, marker), Value::Prefix(text, true, _) if text.contains("SELECT 1")),
            "{marker}"
        );
    }
}

#[test]
fn unary_zero_indices_match_canonical_arguments_slot_zero_for_reads_and_writes() {
    let values = outputs("helper-tracing-unary-argument-index");
    for (marker, sql) in [
        ("positive-zero-read", "SELECT 1"),
        ("negative-zero-read", "SELECT 2"),
        ("positive-zero-write", "SELECT 3"),
        ("negative-zero-write", "SELECT 4"),
    ] {
        assert!(
            matches!(event(&values, marker), Value::Prefix(value, true, None) if value == sql),
            "{marker}"
        );
    }
}

#[test]
fn effect_references_preserve_callbacks_mutations_and_single_operand_evaluation() {
    let values = outputs("reference-fragment-effects");
    for (marker, text) in [
        ("comparison operand effect", "SELECT 1"),
        ("first object callback", "SELECT 2"),
        ("second object callback", "SELECT 3"),
        ("scalar comparison bind", "sql_placeholder_1"),
        ("mutation comparison bind", "sql_placeholder_1"),
        ("interpolation mutation effect", "SELECT 5"),
        ("dynamic-index operand effect", "SELECT 6"),
        ("computed-object key effect", "SELECT 7"),
    ] {
        assert!(
            matches!(event(&values, marker), Value::Prefix(sql, true, _) if crate::codebase::postgres::embedded::publish_placeholders_with_positions(sql.clone()).0 == text),
            "{marker}"
        );
        assert_eq!(
            values
                .iter()
                .filter(|(sql, _)| sql.contains(marker))
                .count(),
            1
        );
    }
    assert!(matches!(
        event(&values, "object builder escape"),
        Value::Unknown
    ));
    for marker in [
        "dynamic-index fragment candidate",
        "object-property fragment candidate",
    ] {
        assert!(matches!(event(&values, marker), Value::Prefix(text, false, _) if text.is_empty()));
        assert_eq!(
            values
                .iter()
                .filter(|(sql, _)| sql.contains(marker))
                .count(),
            1
        );
    }
    assert_eq!(values.len(), 11);
}

