use no_mistakes::codebase::postgres::{
    extract_executed_postgres_sql_with_imports, parse_postgres_sql, EmbeddedSqlOptions,
    ExecutedPostgresSource, ExecutedPostgresSql, TrustedSqlTag,
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
fn whole_statement_helper_branches_and_collections() {
    let rows = extract("control-flow-collections.ts");
    for (annotation, count) in [
        ("branch/read", 2),
        ("collections/map", 1),
        ("collections/conditional", 2),
        ("collections/append", 1),
        ("collections/empty-append", 1),
        ("branch/mixed", 2),
    ] {
        let variants: Vec<_> = rows
            .iter()
            .filter(|row| row.annotation.as_deref() == Some(annotation))
            .collect();
        assert_eq!(variants.len(), count, "{annotation}: {rows:?}");
        for variant in variants {
            assert!(
                variant.complete || variant.only_value_placeholders,
                "{variant:?}"
            );
            assert!(variant.incomplete_reason.is_none(), "{variant:?}");
            assert!(
                parse_postgres_sql(variant.sql_text.as_ref().unwrap()).is_ok(),
                "{variant:?}"
            );
        }
    }
}

#[test]
fn unknown_runtime_collections_and_effects_never_prove_reads() {
    let rows = extract("control-flow-collections.ts");
    let unknown: Vec<_> = rows
        .iter()
        .filter(|row| !row.complete && !row.only_value_placeholders)
        .collect();
    assert_eq!(unknown.len(), 4, "{rows:?}");
    for row in unknown {
        assert!(row.incomplete_reason.is_some(), "{row:?}");
    }
}

#[test]
fn shared_variant_limit_and_every_alternative_are_required() {
    let rows = extract("control-flow-limits.ts");
    let at_limit: Vec<_> = rows
        .iter()
        .filter(|row| row.annotation.as_deref() == Some("variants/at-limit"))
        .collect();
    assert_eq!(
        at_limit.len(),
        no_mistakes::codebase::postgres::MAX_EMBEDDED_SQL_VARIANTS
    );
    assert!(at_limit.iter().all(|row| row.complete));
    assert!(rows
        .iter()
        .any(|row| row.incomplete_reason.as_deref() == Some("variant-limit")));
    for row in rows.iter().filter(|row| {
        row.annotation.as_deref() != Some("variants/at-limit")
            && row.annotation.as_deref() != Some("append/mixed")
    }) {
        assert!(!row.complete && !row.only_value_placeholders, "{row:?}");
        assert!(row.incomplete_reason.is_some(), "{row:?}");
    }
    let mixed: Vec<_> = rows
        .iter()
        .filter(|row| row.annotation.as_deref() == Some("append/mixed"))
        .collect();
    assert_eq!(mixed.len(), 2, "{rows:?}");
    assert!(mixed.iter().all(|row| row.complete
        && row
            .sql_text
            .as_ref()
            .unwrap()
            .contains("SELECT id FROM removed")));
    assert!(mixed.iter().any(|row| row
        .sql_text
        .as_ref()
        .unwrap()
        .contains("DELETE FROM records")));
}

// Consumers must inspect every parsed alternative, including nested statements;
// SELECT/WITH prefix tests would wrongly accept the modifying CTE in this fixture.
fn read_statement_shape(rows: &[&ExecutedPostgresSql]) -> bool {
    use sqlparser::ast::{Query, Select, Statement, Visit, Visitor};
    use std::ops::ControlFlow;
    struct Reads;
    impl Visitor for Reads {
        type Break = ();
        fn pre_visit_statement(&mut self, statement: &Statement) -> ControlFlow<()> {
            if matches!(statement, Statement::Query(_)) {
                ControlFlow::Continue(())
            } else {
                ControlFlow::Break(())
            }
        }
        fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
            if query.locks.is_empty() {
                ControlFlow::Continue(())
            } else {
                ControlFlow::Break(())
            }
        }
        fn pre_visit_select(&mut self, select: &Select) -> ControlFlow<()> {
            if select.into.is_none() {
                ControlFlow::Continue(())
            } else {
                ControlFlow::Break(())
            }
        }
    }
    !rows.is_empty()
        && rows.iter().all(|row| {
            if !row.complete && !row.only_value_placeholders {
                return false;
            }
            row.sql_text
                .as_ref()
                .and_then(|text| parse_postgres_sql(text).ok())
                .is_some_and(|statements| {
                    !statements.is_empty()
                        && matches!(statements.visit(&mut Reads), ControlFlow::Continue(()))
                })
        })
}

#[test]
fn read_classification_requires_all_parsed_variants_and_rejects_nested_effects() {
    let rows = extract("control-flow-collections.ts");
    let group = |annotation| {
        rows.iter()
            .filter(|row| row.annotation.as_deref() == Some(annotation))
            .collect::<Vec<_>>()
    };
    assert!(read_statement_shape(&group("branch/read")));
    assert!(read_statement_shape(&group("collections/map")));
    assert!(!read_statement_shape(&group("branch/mixed")));
    let rows = extract("control-flow-limits.ts");
    let mixed = rows
        .iter()
        .filter(|row| row.annotation.as_deref() == Some("append/mixed"))
        .collect::<Vec<_>>();
    assert!(!read_statement_shape(&mixed));
    assert!(!read_statement_shape(
        &rows.iter().filter(|row| !row.complete).collect::<Vec<_>>()
    ));
}

#[test]
fn opaque_collection_callbacks_and_shapes_keep_explicit_blockers() {
    let rows = extract("control-flow-effects.ts");
    for row in rows.iter().filter(|row| {
        !matches!(
            row.annotation.as_deref(),
            Some(
                "effects/live-elements"
                    | "effects/comparisons"
                    | "effects/mapped-branches"
                    | "effects/unrelated-deletion"
                    | "effects/false-branch"
            )
        )
    }) {
        assert!(!row.complete && !row.only_value_placeholders, "{row:?}");
        assert!(row.incomplete_reason.is_some(), "{row:?}");
    }
    let live = rows
        .iter()
        .find(|row| row.annotation.as_deref() == Some("effects/live-elements"))
        .unwrap();
    assert!(live.complete, "{live:?}");
    assert!(live
        .sql_text
        .as_ref()
        .unwrap()
        .contains("DELETE FROM records"));
    assert!(!read_statement_shape(&[live]));
    let comparisons = rows
        .iter()
        .find(|row| row.annotation.as_deref() == Some("effects/comparisons"))
        .unwrap();
    assert_eq!(
        comparisons.sql_text.as_deref(),
        Some("/* effects/comparisons */ SELECT 1,2")
    );
}

#[test]
fn mapped_element_alternatives_produce_whole_statement_variants() {
    let rows = extract("control-flow-effects.ts");
    let variants = rows
        .iter()
        .filter(|row| row.annotation.as_deref() == Some("effects/mapped-branches"))
        .collect::<Vec<_>>();
    assert_eq!(variants.len(), 4, "{rows:?}");
    assert!(read_statement_shape(&variants));
}

#[test]
fn collection_deletion_is_scoped_and_static_false_branches_do_not_invent_writes() {
    let rows = extract("control-flow-effects.ts");
    for name in ["effects/unrelated-deletion", "effects/false-branch"] {
        let variants = rows
            .iter()
            .filter(|row| row.annotation.as_deref() == Some(name))
            .collect::<Vec<_>>();
        assert_eq!(variants.len(), 1, "{rows:?}");
        assert!(read_statement_shape(&variants), "{variants:?}");
    }
}

#[test]
fn opaque_operand_effects_survive_final_collection_and_scalar_values() {
    let rows = extract("control-flow-operand-effects.ts");
    assert_eq!(rows.len(), 14, "{rows:?}");
    let safe = rows
        .iter()
        .filter(|row| row.complete || row.only_value_placeholders)
        .collect::<Vec<_>>();
    assert_eq!(safe.len(), 2, "{rows:?}");
    assert!(read_statement_shape(&safe));
    for row in rows
        .iter()
        .filter(|row| !row.complete && !row.only_value_placeholders)
    {
        assert!(row.sql_text.is_none(), "{row:?}");
        assert!(row.incomplete_reason.is_some(), "{row:?}");
    }
}

#[test]
fn sparse_shapes_do_not_choose_safe_branches_and_literal_elements_refresh_builders() {
    let rows = extract("control-flow-sparse-snapshots.ts");
    assert_eq!(rows.len(), 2, "{rows:?}");
    assert!(
        !rows[0].complete && !rows[0].only_value_placeholders,
        "{rows:?}"
    );
    assert_eq!(
        rows[0].incomplete_reason.as_deref(),
        Some("sparse-or-opaque-collection-shape")
    );
    assert!(rows[1].complete, "{rows:?}");
    assert!(rows[1]
        .sql_text
        .as_ref()
        .unwrap()
        .contains("DELETE FROM records"));
    assert!(!read_statement_shape(&[&rows[1]]));
}

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
    let rows = extract("control-flow-executor-callee-effects.ts");
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
