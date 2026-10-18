use super::{extract, ExecutedPostgresSql};

#[test]
fn unvisited_branch_local_callbacks_never_certify_a_lexical_prefix() {
    let rows = extract("control-flow-branch-local-callback.ts");
    assert_eq!(rows.len(), 23, "{rows:?}");
    for row in rows
        .iter()
        .filter(|row| matches!(row.line, 51 | 58 | 65) || row.line >= 106)
    {
        assert!(!row.complete && !row.only_value_placeholders, "{rows:?}");
    }
    // Retaining a callback does not invoke it. Its lexical SQL is still opaque
    // after an untyped helper, while the untouched outer query stays complete.
    for row in rows.iter().filter(|row| row.line > 70 && row.line < 105) {
        assert_eq!(row.complete, row.line == 100, "{rows:?}");
        if row.line == 100 {
            continue;
        }
        assert_eq!(
            row.incomplete_reason.as_deref(),
            Some("unresolved-sql-fragment"),
            "{row:?}"
        );
    }
    let callback = rows.iter().find(|row| row.line == 8).unwrap();
    assert!(
        !callback.complete && !callback.only_value_placeholders,
        "{rows:?}"
    );
    let reachable = rows.iter().find(|row| row.line == 23).unwrap();
    assert!(reachable.complete, "{rows:?}");
    assert_eq!(reachable.sql_text.as_deref(), Some("SELECT 1"));
    let uninvoked = rows.iter().find(|row| row.line == 34).unwrap();
    // The lexical fallback cannot certify pre-mutation SQL as complete either.
    assert!(!uninvoked.complete, "{rows:?}");
    assert_eq!(
        uninvoked.incomplete_reason.as_deref(),
        Some("unresolved-sql-fragment")
    );
    let hoisted = rows.iter().find(|row| row.line == 43).unwrap();
    assert!(
        !hoisted.complete && !hoisted.only_value_placeholders,
        "{rows:?}"
    );
}

#[test]
fn primitive_projection_never_proves_an_unresolved_intrinsic_effect_pure() {
    let rows = extract("control-flow-primitive-opaque-intrinsics.ts");
    assert_eq!(rows.len(), 4, "{rows:?}");
    for row in &rows {
        assert_eq!(row.complete, row.line == 18, "{rows:?}");
    }
}

#[test]
fn handled_alternative_updates_cannot_revive_an_earlier_builder_snapshot() {
    let rows = extract("control-flow-handled-alternative-updates.ts");
    assert_eq!(rows.len(), 8, "{rows:?}");
    for row in &rows {
        // Assignment operands remain unmodeled; this fix adds no rebind precision.
        assert_eq!(row.complete, row.line != 28, "{rows:?}");
    }
    for line in [9, 32] {
        let variants = rows
            .iter()
            .filter(|row| row.line == line)
            .collect::<Vec<_>>();
        assert_eq!(variants.len(), 2, "{rows:?}");
        assert!(
            variants
                .iter()
                .any(|row| row.sql_text.as_ref().unwrap().contains("DELETE")),
            "{rows:?}"
        );
    }
    for line in [13, 17, 23] {
        let row = rows.iter().find(|row| row.line == line).unwrap();
        assert_eq!(row.sql_text.as_deref(), Some("SELECT 1"), "{rows:?}");
    }
}

#[test]
fn failed_callback_container_aliases_keep_only_actual_invocation_metadata() {
    let rows = extract("control-flow-callback-container-aliases.ts");
    assert_eq!(rows.len(), 9, "{rows:?}");
    assert_callback_rows(&rows, &[6, 14, 41, 50], &[34, 38, 46]);
}

#[test]
fn sparse_failed_arrays_retain_callback_positions_without_dense_allocation() {
    let rows = extract("control-flow-sparse-failed-array.ts");
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert!(!rows[0].complete, "{rows:?}");
}

#[test]
fn opaque_object_literals_keep_inert_callback_references_for_actual_calls() {
    let rows = extract("control-flow-opaque-object-callbacks.ts");
    assert_eq!(rows.len(), 8, "{rows:?}");
    assert_callback_rows(&rows, &[6, 13, 20, 27, 49], &[45]);
}

#[test]
fn conditional_failed_bindings_keep_actual_callback_identity() {
    let rows = extract("control-flow-conditional-callback-bindings.ts");
    assert_eq!(rows.len(), 9, "{rows:?}");
    assert_callback_rows(&rows, &[6, 12, 31, 37, 43, 49], &[27]);
}

#[test]
fn poisoned_arguments_preserve_actual_callback_identity() {
    let rows = extract("control-flow-poisoned-arguments-callbacks.ts");
    assert_eq!(rows.len(), 5, "{rows:?}");
    assert_callback_rows(&rows, &[8, 31], &[24]);
}

fn assert_callback_rows(rows: &[ExecutedPostgresSql], invoked: &[u32], pure: &[u32]) {
    // Actual invocations retain their evaluator metadata. Uninvoked bodies
    // expose only a conservative lexical fallback, never the stale SELECT.
    for row in rows {
        assert_eq!(row.complete, pure.contains(&row.line), "{rows:?}");
        let reason = if pure.contains(&row.line) {
            None
        } else if invoked.contains(&row.line) {
            Some("branch-local-binding")
        } else {
            Some("unresolved-sql-fragment")
        };
        assert_eq!(row.incomplete_reason.as_deref(), reason, "{row:?}");
    }
}
