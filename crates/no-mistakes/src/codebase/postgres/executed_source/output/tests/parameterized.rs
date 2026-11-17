use super::extract;

#[test]
fn generated_value_binds_preserve_parseable_shape_without_claiming_concrete_values() {
    let statements = extract("parameterized-bind-heads.ts");
    assert_eq!(statements.len(), 3, "{statements:?}");
    for statement in &statements[..2] {
        assert!(!statement.complete, "{statement:?}");
        assert!(statement.only_value_placeholders, "{statement:?}");
        assert_eq!(statement.incomplete_reason, None, "{statement:?}");
    }
    assert_eq!(
        statements[0].sql_text.as_deref(),
        Some("SELECT $1, $2 FROM generate_series(1, $3) LIMIT $4")
    );
    assert_eq!(
        statements[1].sql_text.as_deref(),
        Some("SELECT $1 WHERE $2 = $3")
    );
    assert!(!statements[2].complete);
    assert!(!statements[2].only_value_placeholders);
}

#[test]
fn value_bind_provenance_rejects_missing_or_misaligned_positions() {
    for (text, positions) in [
        (None, vec![(1, 1)]),
        (Some("$1"), vec![]),
        (Some("$1"), vec![(2, 1)]),
        (Some("sql_placeholder_1"), vec![(1, 1)]),
        (Some("$1\nsql_placeholder_2"), vec![(1, 1), (2, 1)]),
    ] {
        assert!(!super::super::only_value_binds(text, &positions));
    }
    assert!(super::super::only_value_binds(
        Some("SELECT\n$1"),
        &[(2, 1)]
    ));
}

#[test]
fn import_projection_preserves_parameterizing_tag_bind_ordinals() {
    let statements = super::extract_with_imports("parameterized-bind-heads.ts", true);
    for (statement, expected) in statements.iter().take(2).zip([
        "SELECT $1, $2 FROM generate_series(1, $3) LIMIT $4",
        "SELECT $1 WHERE $2 = $3",
    ]) {
        assert_eq!(statement.sql_text.as_deref(), Some(expected));
        assert!(!statement.complete);
        assert!(statement.only_value_placeholders);
        assert_eq!(statement.incomplete_reason, None);
    }
}

#[test]
fn explicit_positional_parameters_do_not_advance_generated_bind_ordinals() {
    for resolve_imports in [false, true] {
        let statements =
            super::extract_with_imports("parameterized-bind-positional.ts", resolve_imports);
        assert_eq!(statements.len(), 1);
        assert_eq!(
            statements[0].sql_text.as_deref(),
            Some("SELECT $10, $1, $20, $2")
        );
        assert!(statements[0].only_value_placeholders);
    }
}
