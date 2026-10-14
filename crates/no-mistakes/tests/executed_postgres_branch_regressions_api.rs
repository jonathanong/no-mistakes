use no_mistakes::codebase::postgres::{
    extract_executed_postgres_sql_with_imports, EmbeddedSqlOptions, ExecutedPostgresSource,
    ExecutedPostgresSql, TrustedSqlTag,
};
use std::path::PathBuf;

fn extract(name: &str) -> Vec<ExecutedPostgresSql> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/executed-source")
        .join(name);
    extract_executed_postgres_sql_with_imports(
        &[ExecutedPostgresSource {
            file_name: name.into(),
            source: std::fs::read_to_string(path).unwrap(),
        }],
        &EmbeddedSqlOptions::configured("@example/db", &[]).with_trusted_sql_tags(&[
            TrustedSqlTag {
                module: "@example/db".into(),
                name: "sql".into(),
            },
        ]),
        &[],
    )
    .unwrap()
}

#[test]
fn divergent_dense_shapes_do_not_fabricate_a_known_length() {
    let rows = extract("control-flow-branch-shapes.ts");
    let divergent = rows.iter().filter(|row| row.line == 15).collect::<Vec<_>>();
    assert_eq!(divergent.len(), 1, "{rows:?}");
    // A shape failure must block both branches rather than prove the SELECT arm.
    for row in divergent {
        assert!(!row.complete && !row.only_value_placeholders, "{row:?}");
        assert_eq!(
            row.incomplete_reason.as_deref(),
            Some("conditional-collection-shape"),
            "{row:?}"
        );
    }
    let dense = rows
        .iter()
        .filter(|row| row.annotation.as_deref() == Some("shapes/dense"))
        .collect::<Vec<_>>();
    assert_eq!(dense.len(), 1, "{rows:?}");
    assert!(dense[0].complete, "{rows:?}");
    assert_eq!(
        dense[0].sql_text.as_deref(),
        Some("/* shapes/dense */ SELECT 2")
    );
}

#[test]
fn branch_returned_private_functions_remain_safely_incomplete() {
    let rows = extract("control-flow-branch-returned-functions.ts");
    assert_eq!(rows.len(), 2, "{rows:?}");
    // Follow-up #1973 will recover these returned entrypoints. Until then the
    // lexical pre-append prefix must never be certified as the full statement.
    for row in rows {
        assert!(!row.complete && !row.only_value_placeholders, "{row:?}");
        assert_eq!(
            row.incomplete_reason.as_deref(),
            Some("unresolved-sql-fragment")
        );
    }
}

#[test]
fn conditional_shapes_cannot_skip_later_callback_mutations() {
    let rows = extract("control-flow-conditional-callback-shapes.ts");
    assert_eq!(rows.len(), 10, "{rows:?}");
    for row in rows {
        if matches!(row.line, 35 | 44) {
            assert!(row.complete, "{row:?}");
            assert_eq!(
                row.sql_text.as_deref(),
                Some("SELECT 1; DELETE FROM records")
            );
        } else if row.line == 51 {
            // Both push-length alternatives were already recovered before the fix.
            assert!(row.complete, "{row:?}");
        } else {
            assert!(!row.complete && !row.only_value_placeholders, "{row:?}");
        }
    }
}

#[test]
fn operand_failure_cannot_discard_accessor_effects() {
    let rows = extract("control-flow-operand-failure-effects.ts");
    assert_eq!(rows.len(), 7, "{rows:?}");
    assert_eq!(
        rows.iter().filter(|row| row.complete).count(),
        2,
        "{rows:?}"
    );
    for row in rows {
        let pure = matches!(row.line, 26 | 32);
        assert_eq!(row.complete, pure, "{row:?}");
        assert!(!row.only_value_placeholders, "{row:?}");
    }
}

#[test]
fn builder_coercion_observes_later_supported_operand_effects() {
    let rows = extract("control-flow-late-builder-coercion.ts");
    assert_eq!(rows.len(), 5, "{rows:?}");
    for row in &rows {
        if matches!(row.line, 17 | 29) {
            // Builder binary coercion was already conservative before this fix.
            assert!(!row.complete && !row.only_value_placeholders, "{row:?}");
            assert_eq!(
                row.incomplete_reason.as_deref(),
                Some("opaque-binary-coercion")
            );
            continue;
        }
        assert!(row.complete, "{rows:?}");
        let text = if matches!(row.line, 23 | 29) {
            "SELECT 1"
        } else {
            "SELECT 1; DELETE FROM records"
        };
        assert_eq!(row.sql_text.as_deref(), Some(text), "{rows:?}");
    }
}

#[test]
fn ordered_operand_effects_precede_nested_executor_events() {
    let rows = extract("control-flow-ordered-operand-effects.ts");
    assert_eq!(rows.len(), 30, "{rows:?}");
    assert_eq!(
        rows.iter().filter(|row| row.complete).count(),
        11,
        "{rows:?}"
    );
    for row in rows {
        let pure = matches!(
            row.line,
            25 | 29 | 57 | 65 | 77 | 82 | 102 | 106 | 112 | 116 | 127
        );
        assert_eq!(row.complete, pure, "{row:?}");
        assert!(!row.only_value_placeholders, "{row:?}");
    }
}

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
    // Dead containers, shadowed aliases and TDZ aliases do not execute bodies.
    for row in rows.iter().filter(|row| row.line > 70 && row.line < 105) {
        assert!(row.complete, "{rows:?}");
        assert_eq!(
            row.kind,
            no_mistakes::codebase::postgres::EmbeddedSqlKind::ImmutableLocal
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
    // Retaining a declaration does not manufacture an evaluator invocation.
    assert!(uninvoked.complete, "{rows:?}");
    assert_eq!(
        uninvoked.kind,
        no_mistakes::codebase::postgres::EmbeddedSqlKind::ImmutableLocal
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
    for row in &rows {
        assert_eq!(
            row.complete,
            !matches!(row.line, 6 | 14 | 41 | 50),
            "{rows:?}"
        );
    }
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
    // Uninvoked and statically false callback bodies retain lexical completeness.
    for row in &rows {
        assert_eq!(row.complete, matches!(row.line, 34 | 41 | 45), "{rows:?}");
    }
}
