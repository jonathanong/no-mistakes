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
    assert_eq!(rows.len(), 13, "{rows:?}");
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
