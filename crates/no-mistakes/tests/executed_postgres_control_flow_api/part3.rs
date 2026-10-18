use super::*;

#[test]
fn unsupported_helpers_with_appends_cannot_revive_safe_sql_prefixes() {
    let rows = extract("control-flow-unsupported-append.ts");
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert!(
        !rows[0].complete && !rows[0].only_value_placeholders,
        "{rows:?}"
    );
    assert!(rows[0].sql_text.is_none(), "{rows:?}");
    assert!(rows[0].incomplete_reason.is_some(), "{rows:?}");
}

#[test]
fn unsupported_nested_branch_effects_cannot_certify_caller_builders() {
    let rows = extract("control-flow-nested-unsupported.ts");
    assert_eq!(rows.len(), 3, "{rows:?}");
    for line in [23, 28] {
        let row = rows.iter().find(|row| row.line == line).unwrap();
        assert!(!row.complete && !row.only_value_placeholders, "{row:?}");
        assert!(row.sql_text.is_none(), "{row:?}");
        assert!(row.incomplete_reason.is_some(), "{row:?}");
    }
    let row = rows
        .iter()
        .find(|row| row.annotation.as_deref() == Some("nested/unreachable"))
        .unwrap();
    assert!(row.complete, "{row:?}");
}

#[test]
fn unreachable_var_declarations_still_shadow_captured_sql() {
    let rows = extract("control-flow-hoisted-branches.ts");
    assert_eq!(rows.len(), 2, "{rows:?}");
    let shadowed = rows.iter().find(|row| row.line == 6).unwrap();
    assert!(
        !shadowed.complete && !shadowed.only_value_placeholders,
        "{shadowed:?}"
    );
    let unchanged = rows.iter().find(|row| row.line == 11).unwrap();
    assert!(unchanged.complete, "{unchanged:?}");
}

#[test]
fn opaque_array_callbacks_remain_safely_incomplete() {
    let rows = extract("control-flow-array-callbacks.ts");
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert!(
        !rows[0].complete && !rows[0].only_value_placeholders,
        "{rows:?}"
    );
    assert!(
        rows[0].sql_text.is_none(),
        "observed opaque callback discards the earlier fragment: {rows:?}"
    );
    assert_eq!(
        rows[0].incomplete_reason.as_deref(),
        Some("unresolved-value-or-effect"),
        "{rows:?}"
    );
}

#[test]
fn direct_unresolved_executor_operands_cannot_certify_earlier_builders() {
    let rows = extract("control-flow-executor-direct-operand.ts");
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert!(
        !rows[0].complete && !rows[0].only_value_placeholders,
        "{rows:?}"
    );
    assert_eq!(
        rows[0].incomplete_reason.as_deref(),
        Some("opaque-executor-operand-effect")
    );
}

#[test]
fn mapped_builder_slots_include_later_iteration_mutations() {
    let rows = extract("control-flow-map-builder-alias.ts");
    assert_eq!(rows.len(), 3, "{rows:?}");
    assert!(
        rows.iter().all(|row| {
            !row.complete
                || row
                    .sql_text
                    .as_deref()
                    .is_some_and(|text| text.contains("DELETE FROM records"))
        }),
        "{rows:?}"
    );
}

#[test]
fn executor_receiver_effects_cannot_certify_exported_builders() {
    // This fixture probes effects of arbitrary receiver expressions, so its
    // member-query selection is deliberately broad rather than module scoped.
    let options = EmbeddedSqlOptions::configured("", &["query".into()]).with_trusted_sql_tags(&[
        TrustedSqlTag {
            module: "@example/db".into(),
            name: "sql".into(),
        },
    ]);
    let rows = extract_with_options("control-flow-executor-callee-effects.ts", &options);
    assert_eq!(rows.len(), 13, "{rows:?}");
    assert_eq!(
        rows.iter().filter(|row| row.complete).count(),
        6,
        "{rows:?}"
    );
    for row in rows {
        let pure = matches!(row.line, 21 | 25 | 28 | 31 | 40 | 43);
        assert_eq!(row.complete, pure, "{row:?}");
        if pure {
            assert!(!row.only_value_placeholders, "{row:?}");
            assert!(row.incomplete_reason.is_none(), "{row:?}");
            let (kind, text, annotation) = match row.line {
                31 => (
                    no_mistakes::codebase::postgres::EmbeddedSqlKind::Inline,
                    "/* callees/optional */ SELECT 1",
                    Some("callees/optional"),
                ),
                40 => (
                    no_mistakes::codebase::postgres::EmbeddedSqlKind::Inline,
                    "/* callees/optional-call */ SELECT 1",
                    Some("callees/optional-call"),
                ),
                43 => (
                    no_mistakes::codebase::postgres::EmbeddedSqlKind::Inline,
                    "/* callees/combined */ SELECT 1",
                    Some("callees/combined"),
                ),
                _ => (
                    no_mistakes::codebase::postgres::EmbeddedSqlKind::Composed,
                    "SELECT 1",
                    None,
                ),
            };
            assert_eq!(row.kind, kind);
            assert_eq!(row.sql_text.as_deref(), Some(text));
            assert_eq!(row.annotation.as_deref(), annotation);
        } else {
            assert!(!row.only_value_placeholders, "{row:?}");
            assert_eq!(
                row.incomplete_reason.as_deref(),
                Some("opaque-executor-operand-effect")
            );
        }
    }
}
